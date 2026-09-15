//! Artists on MusicBrainz: search, lookup, and discography.
//!
//! Three requests, each behind the same one-per-second gate as album lookups:
//!
//! - **search** (`/artist?query=`) returns name, type, country, active years, MusicBrainz's
//!   note telling same-named artists apart, and tags;
//! - **lookup** (`/artist/{id}?inc=genres+url-rels`) adds proper genres and links;
//! - **browse** (`/release-group?artist={id}&type=album|ep`) lists their albums and EPs,
//!   up to a hundred per request.
//!
//! MusicBrainz has no artist photos and the Cover Art Archive only holds album covers, so
//! an artist's picture is the cover of one of their albums. ADR 0007 has the reasoning.

use super::coverart::small_cover_url;
use super::musicbrainz::{
    collect_genres, escape_lucene, is_real_country, read_candidate, searchable_words, urlencode, API,
};
use super::Resolver;
use crate::domain::{AlbumCandidate, ArtistCandidate, ArtistLink, ArtistMetadata};
use serde_json::Value;

/// Browse pages fetched for one discography, at a hundred albums each. Enough for every
/// artist but a handful of the most prolific, and a bound on how long the page can take.
const MAX_BROWSE_PAGES: usize = 3;
const BROWSE_PAGE: usize = 100;

/// The links worth showing, in the order they are shown. MusicBrainz lists dozens of
/// relationship types per artist, most of them for other databases.
const LINK_KINDS: &[&str] = &[
    "official homepage",
    "bandcamp",
    "streaming",
    "free streaming",
    "purchase for download",
    "youtube",
    "youtube music",
    "soundcloud",
    "last.fm",
    "discogs",
    "allmusic",
    "wikidata",
];

impl Resolver {
    /// Artists matching typed text, as MusicBrainz ranks them. None when MusicBrainz cannot
    /// be reached, so that "nothing found" and "could not look" stay distinguishable.
    pub async fn mb_search_artists(&self, query: &str, limit: usize) -> Option<Vec<ArtistCandidate>> {
        let Some(lucene) = artist_query(query) else {
            return Some(Vec::new());
        };
        let url = format!("{API}/artist?query={}&fmt=json&limit={limit}", urlencode(&lucene));
        let body = self.mb_get(&url).await?;
        Some(
            body.get("artists")
                .and_then(Value::as_array)
                .map(|hits| hits.iter().filter_map(read_artist_candidate).collect())
                .unwrap_or_default(),
        )
    }

    /// An artist's details, genres and links. None when the request fails.
    pub async fn mb_artist(&self, id: &str) -> Option<ArtistMetadata> {
        let url = format!("{API}/artist/{id}?inc=genres+url-rels&fmt=json");
        let body = self.mb_get(&url).await?;
        read_artist(&body)
    }

    /// Every album and EP credited to an artist, in the order MusicBrainz returns them.
    ///
    /// None when the first page cannot be fetched. A later page failing returns what came
    /// before it: most of a discography is more use than an error.
    pub async fn mb_artist_release_groups(&self, id: &str) -> Option<Vec<Value>> {
        let mut groups = Vec::new();
        for page in 0..MAX_BROWSE_PAGES {
            let offset = page * BROWSE_PAGE;
            let url = format!(
                "{API}/release-group?artist={id}&type=album%7Cep&limit={BROWSE_PAGE}&offset={offset}&fmt=json"
            );
            let Some(body) = self.mb_get(&url).await else {
                return (page > 0).then_some(groups);
            };
            let batch = body
                .get("release-groups")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let total = body.get("release-group-count").and_then(Value::as_u64).unwrap_or(0) as usize;
            let fetched = batch.len();
            groups.extend(batch);
            if fetched < BROWSE_PAGE || groups.len() >= total {
                break;
            }
        }
        Some(groups)
    }
}

/// Whether text is shaped like a MusicBrainz id. Ids arrive from the frontend and go into a
/// URL path, so anything else is refused before a request is built.
pub fn is_mbid(id: &str) -> bool {
    id.len() == 36
        && id.char_indices().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// The search for typed text. An exact name scores highest; otherwise any of the words may
/// match the name, a sort name or an alias, and MusicBrainz's score orders the rest.
fn artist_query(text: &str) -> Option<String> {
    let words = searchable_words(text);
    if words.is_empty() {
        return None;
    }
    let escaped: Vec<String> = words.iter().map(|w| escape_lucene(w)).collect();
    Some(format!("artist:\"{}\"^3 OR {}", escaped.join(" "), escaped.join(" ")))
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// The year from a MusicBrainz partial date: "1999", "1999-08" or "1999-08-25".
fn year(date: Option<&Value>) -> Option<i32> {
    date?.as_str()?.get(0..4)?.parse().ok()
}

/// The years an artist was active. An end year only for an artist that has ended.
fn life_span(artist: &Value) -> (Option<i32>, Option<i32>) {
    let span = artist.get("life-span");
    let begin = year(span.and_then(|s| s.get("begin")));
    let end = year(span.and_then(|s| s.get("end")));
    (begin, end)
}

/// Where an artist is from: their country code, or their area's name where there is no code,
/// as for an album's country.
fn country(artist: &Value) -> Option<String> {
    text(artist, "country")
        .filter(|c| is_real_country(c))
        .or_else(|| artist.get("area").and_then(|a| text(a, "name")))
}

fn read_artist_candidate(hit: &Value) -> Option<ArtistCandidate> {
    let (begin_year, end_year) = life_span(hit);
    let mut tags: Vec<(i64, String)> = hit
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter()
                .filter_map(|t| Some((t.get("count").and_then(Value::as_i64).unwrap_or(0), text(t, "name")?)))
                .collect()
        })
        .unwrap_or_default();
    tags.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

    Some(ArtistCandidate {
        musicbrainz_artist_id: text(hit, "id")?,
        name: text(hit, "name")?,
        disambiguation: text(hit, "disambiguation"),
        artist_type: text(hit, "type"),
        country: country(hit),
        begin_year,
        end_year,
        tags: tags.into_iter().take(4).map(|(_, name)| name).collect(),
    })
}

/// An artist lookup, without the picture, which needs their discography.
fn read_artist(body: &Value) -> Option<ArtistMetadata> {
    let (begin_year, end_year) = life_span(body);
    Some(ArtistMetadata {
        musicbrainz_artist_id: text(body, "id")?,
        name: text(body, "name")?,
        disambiguation: text(body, "disambiguation"),
        artist_type: text(body, "type"),
        country: country(body),
        begin_year,
        end_year,
        image_url: None,
        genres: collect_genres(body.get("genres")),
        links: read_links(body),
    })
}

/// The links worth showing, in `LINK_KINDS` order. A link MusicBrainz marks as ended is a
/// dead page, and the same address listed under two types is shown once.
fn read_links(body: &Value) -> Vec<ArtistLink> {
    let mut links: Vec<(usize, ArtistLink)> = body
        .get("relations")
        .and_then(Value::as_array)
        .map(|rels| {
            rels.iter()
                .filter(|r| r.get("ended").and_then(Value::as_bool) != Some(true))
                .filter_map(|r| {
                    let kind = r.get("type")?.as_str()?;
                    let rank = LINK_KINDS.iter().position(|k| *k == kind)?;
                    let url = r.get("url")?.get("resource")?.as_str()?;
                    (url.starts_with("https://") || url.starts_with("http://")).then(|| {
                        (rank, ArtistLink { kind: kind.to_string(), url: url.to_string() })
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    links.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.url.cmp(&b.1.url)));

    let mut seen = std::collections::HashSet::new();
    links
        .into_iter()
        .map(|(_, link)| link)
        .filter(|link| seen.insert(link.url.clone()))
        .collect()
}

fn first_release_date(group: &Value) -> Option<&str> {
    group
        .get("first-release-date")
        .and_then(Value::as_str)
        .filter(|d| !d.is_empty())
}

fn is_plain(group: &Value, primary: &str) -> bool {
    group.get("primary-type").and_then(Value::as_str) == Some(primary)
        && group
            .get("secondary-types")
            .and_then(Value::as_array)
            .map_or(true, |t| t.is_empty())
}

/// A discography as the page shows it: oldest first, undated albums last.
pub fn discography(groups: &[Value]) -> Vec<AlbumCandidate> {
    let mut sorted: Vec<&Value> = groups.iter().collect();
    sorted.sort_by(|a, b| {
        match (first_release_date(a), first_release_date(b)) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| text(a, "title").cmp(&text(b, "title")))
    });
    sorted.into_iter().filter_map(read_candidate).collect()
}

/// The cover that stands in for an artist's photo: their first studio album's, or failing
/// that their first EP's, or failing that anything they have.
///
/// A studio album is the likeliest to have a cover in the archive. Live albums and
/// compilations often do not, and a bootleg almost never.
pub fn pick_image(groups: &[Value]) -> Option<String> {
    let dated_first = |kind: &str| {
        groups
            .iter()
            .filter(|g| is_plain(g, kind))
            .min_by(|a, b| match (first_release_date(a), first_release_date(b)) {
                (Some(x), Some(y)) => x.cmp(y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
    };
    dated_first("Album")
        .or_else(|| dated_first("EP"))
        .or_else(|| groups.first())
        .and_then(|g| text(g, "id"))
        .map(|id| small_cover_url(&id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn group(id: &str, date: &str, primary: &str, secondary: &[&str]) -> Value {
        json!({"id": id, "title": id, "first-release-date": date,
               "primary-type": primary, "secondary-types": secondary})
    }

    #[test]
    fn recognises_musicbrainz_ids_and_nothing_else() {
        assert!(is_mbid("24e1b53c-3085-4581-8472-0b0088d2508c"));
        assert!(!is_mbid("24e1b53c30854581847200b0088d2508cxx"), "no dashes");
        assert!(!is_mbid("24e1b53c-3085-4581-8472-0b0088d2508g"), "not hex");
        assert!(!is_mbid("../../release-group?query=x&fmt=json12"));
        assert!(!is_mbid(""));
    }

    #[test]
    fn the_search_boosts_the_exact_name_and_escapes_syntax() {
        assert_eq!(
            artist_query("avenged sevenfold").as_deref(),
            Some(r#"artist:"avenged sevenfold"^3 OR avenged sevenfold"#)
        );
        assert_eq!(artist_query("AC/DC").as_deref(), Some(r#"artist:"AC\/DC"^3 OR AC\/DC"#));
        assert_eq!(artist_query(" - "), None);
    }

    #[test]
    fn reads_a_search_hit() {
        let hit = json!({
            "id": "24e1b53c-3085-4581-8472-0b0088d2508c", "name": "Avenged Sevenfold",
            "type": "Group", "country": "US", "disambiguation": "American metal band",
            "life-span": {"begin": "1999", "ended": null},
            "tags": [{"name": "metalcore", "count": 2}, {"name": "heavy metal", "count": 7},
                     {"name": "american", "count": 1}, {"name": "metal", "count": 7},
                     {"name": "rock", "count": 0}]
        });
        let c = read_artist_candidate(&hit).unwrap();
        assert_eq!(c.name, "Avenged Sevenfold");
        assert_eq!(c.artist_type.as_deref(), Some("Group"));
        assert_eq!(c.country.as_deref(), Some("US"));
        assert_eq!(c.begin_year, Some(1999));
        assert_eq!(c.end_year, None);
        assert_eq!(c.tags, ["heavy metal", "metal", "metalcore", "american"], "most voted, four");
    }

    #[test]
    fn an_artist_without_a_country_code_takes_their_area() {
        let hit = json!({"id": "x", "name": "Radiohead", "country": "XE", "area": {"name": "England"},
                         "life-span": {"begin": "1991-01", "end": "2030-05-01"}});
        let c = read_artist_candidate(&hit).unwrap();
        assert_eq!(c.country.as_deref(), Some("England"));
        assert_eq!((c.begin_year, c.end_year), (Some(1991), Some(2030)));
    }

    #[test]
    fn a_hit_without_an_id_or_name_is_skipped() {
        assert!(read_artist_candidate(&json!({"name": "x"})).is_none());
        assert!(read_artist_candidate(&json!({"id": "x", "name": "  "})).is_none());
    }

    #[test]
    fn keeps_useful_live_links_in_order_once_each() {
        let body = json!({"id": "x", "name": "X", "relations": [
            {"type": "discogs", "url": {"resource": "https://www.discogs.com/artist/1"}},
            {"type": "myspace", "url": {"resource": "https://myspace.com/x"}},
            {"type": "bandcamp", "url": {"resource": "https://x.bandcamp.com/"}},
            {"type": "official homepage", "ended": true, "url": {"resource": "https://old.example"}},
            {"type": "official homepage", "url": {"resource": "https://x.example"}},
            {"type": "purchase for download", "url": {"resource": "https://x.bandcamp.com/"}},
            {"type": "streaming", "url": {"resource": "ftp://nope"}}
        ]});
        let links = read_artist(&body).unwrap().links;
        let urls: Vec<&str> = links.iter().map(|l| l.url.as_str()).collect();
        assert_eq!(urls, ["https://x.example", "https://x.bandcamp.com/", "https://www.discogs.com/artist/1"]);
        assert_eq!(links[1].kind, "bandcamp");
    }

    #[test]
    fn a_discography_runs_oldest_first_with_undated_albums_last() {
        let groups = [
            group("nightmare", "2010-07-23", "Album", &[]),
            group("undated", "", "Album", &["Live"]),
            group("waking", "2003-08-26", "Album", &[]),
            group("warmness", "2001-04-10", "EP", &[]),
        ];
        let order: Vec<String> = discography(&groups)
            .into_iter()
            .map(|a| a.musicbrainz_release_group_id)
            .collect();
        assert_eq!(order, ["warmness", "waking", "nightmare", "undated"]);
    }

    #[test]
    fn the_picture_is_the_first_studio_album_cover() {
        let groups = [
            group("live", "1999", "Album", &["Live"]),
            group("ep", "2001", "EP", &[]),
            group("second", "2005", "Album", &[]),
            group("debut", "2003", "Album", &[]),
        ];
        assert_eq!(pick_image(&groups), Some(small_cover_url("debut")));

        let eps_only = [group("live", "1999", "Album", &["Live"]), group("ep", "2001", "EP", &[])];
        assert_eq!(pick_image(&eps_only), Some(small_cover_url("ep")));

        let odd = [group("live", "1999", "Album", &["Live"])];
        assert_eq!(pick_image(&odd), Some(small_cover_url("live")));
        assert_eq!(pick_image(&[]), None);
    }
}
