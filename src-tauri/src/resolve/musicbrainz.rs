//! MusicBrainz client. Port of `MusicBrainzService`.
//!
//! MusicBrainz is the only provider that needs no account, no key and no approval, which
//! is why it is the backbone rather than an enrichment. In exchange it asks for two
//! things and enforces both: a descriptive `User-Agent`, and no more than one request per
//! second per client.
//!
//! **Everything comes from the release group.** MusicBrainz models an album as a *release
//! group* holding every *release*: each pressing, reissue, regional edition and format.
//! City of Evil is one group of 14 releases. Trecker tracks albums, so it searches groups,
//! looks up groups, and stores the group id as its identity.
//!
//! This replaced a release search, and every metadata bug this module has had came from
//! trusting the pressing that search happened to match: its year was that edition's, its
//! country was wherever it was sold, it often had no genres or cover art of its own, and
//! the same album could be added twice as two different pressings.
//!
//! The group has no country, because where a record was sold is a property of a
//! pressing. The country Trecker shows is where the artist is from.
//!
//! "Release group" is MusicBrainz vocabulary and stays in the code. The interface says
//! album, because that is what one is.

use super::{Resolver, TIMEOUT};
use crate::domain::{AlbumCandidate, ResolvedMetadata};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub(super) const API: &str = "https://musicbrainz.org/ws/2";

/// MusicBrainz country codes that are not countries.
///
/// `XW` is "worldwide" and `XE` is "Europe". Both are correct answers to "where was this
/// released" and useless answers to "where is this music from", which is what a By
/// Country chart is actually asking. Treated as absent so the artist's area can fill in.
pub(super) fn is_real_country(code: &str) -> bool {
    !matches!(code, "XW" | "XE" | "XU" | "ZZ" | "")
}

impl Resolver {
    /// Waits until at least a second has passed since the previous MusicBrainz request.
    ///
    /// The Java slept 1100ms before every call, which is not a rate limiter: it delayed
    /// even the first request of the day, and two concurrent calls still fired together.
    /// Holding the lock across the sleep both serialises requests and skips the wait when
    /// enough time has already elapsed.
    async fn mb_gate(&self) {
        let mut last = self.mb_last_request.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < Duration::from_millis(1100) {
                tokio::time::sleep(Duration::from_millis(1100) - elapsed).await;
            }
        }
        *last = Some(std::time::Instant::now());
    }

    pub(super) async fn mb_get(&self, url: &str) -> Option<Value> {
        let mut attempt = 0;
        loop {
            // Inside the loop: a retry is a new request and must wait its turn like any
            // other, or a burst of retries would break the one-per-second promise.
            self.mb_gate().await;

            let result = self
                .http
                .get(url)
                .header("User-Agent", &self.user_agent)
                .header("Accept", "application/json")
                .timeout(TIMEOUT)
                .send()
                .await;

            let retry_after = match result {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return response.json::<Value>().await.ok();
                    }
                    if !is_retryable(status.as_u16()) || attempt >= 2 {
                        return None;
                    }
                    // 503 arrives as "the server is currently busy, try again later" and
                    // 429 means we asked too fast. Both are worth waiting out; a 404 is
                    // the release genuinely not existing and must not be retried.
                    header_seconds(&response)
                }
                Err(e) if attempt < 2 && (e.is_connect() || e.is_request()) => None,
                Err(_) => return None,
            };

            attempt += 1;
            let backoff = retry_after
                .unwrap_or_else(|| Duration::from_millis(500 * 2u64.pow(attempt)))
                .min(Duration::from_secs(5));
            tokio::time::sleep(backoff).await;
        }
    }

    /// Searches for albums and returns up to `limit` of them, most likely first.
    ///
    /// Returns None only when MusicBrainz cannot be reached, so that "nothing found" and
    /// "could not look" stay distinguishable to the caller.
    pub async fn mb_search_release_groups(
        &self,
        query: &str,
        limit: usize,
    ) -> Option<Vec<AlbumCandidate>> {
        let Some(precise) = search_query(query) else {
            return Some(Vec::new());
        };
        let mut hits = self.release_group_hits(&precise).await?;

        if hits.is_empty() {
            // The precise query needs every word in the artist or the title, so one extra
            // word, such as a year or "and", empties it. A plain search is looser and
            // ranks worse, which is acceptable in a list the person chooses from.
            if let Some(loose) = loose_query(query) {
                hits = self.release_group_hits(&loose).await?;
            }
        }

        Some(
            rank_release_groups(&hits)
                .into_iter()
                .filter_map(read_candidate)
                .take(limit)
                .collect(),
        )
    }

    /// The single most likely album, for paths that cannot ask the person to choose, such
    /// as a pasted Bandcamp link.
    pub async fn mb_search_release_group(&self, query: &str) -> Option<ResolvedMetadata> {
        let best = self.mb_search_release_groups(query, 1).await?.into_iter().next()?;
        (best.artist.is_some() || best.title.is_some()).then(|| ResolvedMetadata {
            artist: best.artist,
            title: best.title,
            release_year: best.release_year,
            musicbrainz_release_group_id: Some(best.musicbrainz_release_group_id),
            ..Default::default()
        })
    }

    /// One page of raw search hits. More than are shown, because ranking reorders them
    /// and the album can sit below a few same-named singles and bootlegs.
    async fn release_group_hits(&self, lucene: &str) -> Option<Vec<Value>> {
        let url = format!(
            "{API}/release-group?query={}&fmt=json&limit=25",
            urlencode(lucene)
        );
        let body = self.mb_get(&url).await?;
        Some(
            body.get("release-groups")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        )
    }

    /// Everything Trecker keeps about an album, from its release group, in one request.
    ///
    /// `artists` is asked for in the same breath because it costs nothing extra and
    /// supplies the artist's own country. Returns None only when the request fails; a
    /// group with no genres and no known artist country is still a usable result.
    pub async fn mb_release_group(&self, id: &str) -> Option<ResolvedMetadata> {
        let url = format!("{API}/release-group/{id}?inc=genres+artists&fmt=json");
        let body = self.mb_get(&url).await?;
        let mut metadata = read_release_group(&body);
        metadata.genres = collect_genres(body.get("genres"));
        metadata.country = artist_country(&body);
        Some(metadata)
    }
}

/// Whether an HTTP status is worth trying again.
///
/// 429 means we went too fast, 503 is MusicBrainz's "currently busy" page, and 502/504
/// are their proxy having a moment. Everything else, 404 above all, is a real answer.
pub(super) fn is_retryable(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

/// Reads a `Retry-After` delay in seconds, when the server bothered to send one.
pub(super) fn header_seconds(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

/// Orders a page of release-group search hits, most likely album first.
///
/// MusicBrainz scores on text alone, so every group whose title matches exactly scores
/// 100 and their order among themselves means nothing. Live results showed why that
/// matters: "The Weeknd - After Hours" ranks the title-track single above the album, and
/// "Metallica - Metallica" ranks a live bootleg, an interview disc and a compilation all
/// above the 1991 album.
///
/// So hits are ordered by score, and among equal scores a plain album beats anything with a
/// secondary type such as live or compilation, which beats every other type. Within that,
/// the group with the most releases wins: the canonical album is the one that has been
/// pressed and reissued, and a bootleg has one release. MusicBrainz's own order breaks what
/// remains. A lower score never moves above a higher one, so a search for an EP or a single
/// still puts it first.
fn rank_release_groups(hits: &[Value]) -> Vec<&Value> {
    let score = |h: &Value| h.get("score").and_then(Value::as_i64).unwrap_or(0);

    let rank = |h: &Value| {
        let album = h.get("primary-type").and_then(Value::as_str) == Some("Album");
        let plain = h
            .get("secondary-types")
            .and_then(Value::as_array)
            .map_or(true, |t| t.is_empty());
        let kind = match (album, plain) {
            (true, true) => 2,
            (true, false) => 1,
            _ => 0,
        };
        let releases = h.get("count").and_then(Value::as_i64).unwrap_or(0);
        (kind, releases)
    };

    let mut ranked: Vec<(usize, &Value)> = hits.iter().enumerate().collect();
    ranked.sort_by(|(ia, a), (ib, b)| {
        score(b)
            .cmp(&score(a))
            .then_with(|| rank(b).cmp(&rank(a)))
            .then_with(|| ia.cmp(ib))
    });
    ranked.into_iter().map(|(_, h)| h).collect()
}

/// The first of `rank_release_groups`, which is what the tie-breaking tests are about.
#[cfg(test)]
fn pick_release_group(hits: &[Value]) -> Option<&Value> {
    rank_release_groups(hits).into_iter().next()
}

/// A search hit as a candidate for the list. None for a hit with no id, which could not be
/// looked up once chosen.
pub(super) fn read_candidate(group: &Value) -> Option<AlbumCandidate> {
    let basics = read_release_group(group);
    let id = basics.musicbrainz_release_group_id?;
    Some(AlbumCandidate {
        album_art_url: super::coverart::small_cover_url(&id),
        musicbrainz_release_group_id: id,
        artist: basics.artist,
        title: basics.title,
        release_year: basics.release_year,
        primary_type: group.get("primary-type").and_then(Value::as_str).map(String::from),
        secondary_types: group
            .get("secondary-types")
            .and_then(Value::as_array)
            .map(|types| types.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default(),
        disambiguation: group
            .get("disambiguation")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(String::from),
    })
}

/// The search for what was typed. None when there is nothing searchable in it.
///
/// `Artist - Title` searches the two fields separately, which is precise and is what the
/// person asked for by typing the separator. Anything else might be a title alone or an
/// artist and title run together, and a title search alone found nothing for the second.
/// So the query scores an exact title highest, and otherwise needs every word to appear in
/// the artist or the title. Checked live: "avenged sevenfold nightmare", "slint
/// spiderland" and "ok computer" all put the right album first with ranking applied, where
/// a plain search ranked the self-titled album and a tribute record above them.
fn search_query(text: &str) -> Option<String> {
    if let Some((artist, title)) = split_artist_title(text) {
        return Some(format!(
            "artist:\"{}\" AND releasegroup:\"{}\"",
            escape_lucene(artist),
            escape_lucene(title)
        ));
    }

    let words = searchable_words(text);
    if words.is_empty() {
        return None;
    }
    let phrase = escape_lucene(&words.join(" "));
    let each = words
        .iter()
        .map(|w| {
            let w = escape_lucene(w);
            format!("(artist:\"{w}\" OR releasegroup:\"{w}\")")
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    Some(format!("releasegroup:\"{phrase}\"^3 OR ({each})"))
}

/// The fallback when the precise search finds nothing: the words alone, in no field, so
/// MusicBrainz searches its default fields and any of them may match.
fn loose_query(text: &str) -> Option<String> {
    let words: Vec<String> = searchable_words(&text.replace(['–', '—'], " "))
        .iter()
        .map(|w| escape_lucene(w))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

/// Hyphen, en dash or em dash with spaces around, since a dash typed on a phone or copied
/// from a web page is rarely the ASCII one. Both sides must have text.
fn split_artist_title(text: &str) -> Option<(&str, &str)> {
    [" - ", " – ", " — "].iter().find_map(|sep| {
        let (artist, title) = text.split_once(sep)?;
        let (artist, title) = (artist.trim(), title.trim());
        (!artist.is_empty() && !title.is_empty()).then_some((artist, title))
    })
}

/// Words with at least one letter or digit. A lone dash or ampersand is not something an
/// artist or title can be required to contain.
pub(super) fn searchable_words(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .collect()
}

/// The fields a search hit and a lookup share.
fn read_release_group(group: &Value) -> ResolvedMetadata {
    ResolvedMetadata {
        artist: first_artist(group).and_then(|c| {
            c.get("name")
                .and_then(Value::as_str)
                .or_else(|| c.get("artist")?.get("name")?.as_str())
                .map(String::from)
        }),
        title: group.get("title").and_then(Value::as_str).map(String::from),
        // Derived by MusicBrainz from the group's earliest release, so this is the album's
        // year and not a reissue's.
        release_year: group
            .get("first-release-date")
            .and_then(Value::as_str)
            .and_then(|d| d.get(0..4))
            .and_then(|y| y.parse().ok()),
        album_art_url: None,
        country: None,
        streaming_links: HashMap::new(),
        genres: Vec::new(),
        musicbrainz_release_group_id: group.get("id").and_then(Value::as_str).map(String::from),
    }
}

/// The artist credit comes in two shapes, depending on whether the album is credited to
/// one artist or a collaboration. Either way the first credit is the one shown.
fn first_artist(group: &Value) -> Option<&Value> {
    group.get("artist-credit")?.as_array()?.first()
}

/// Where the credited artist is from: their country code, or failing that the name of
/// their area, which MusicBrainz uses for places like England that have no ISO code.
fn artist_country(group: &Value) -> Option<String> {
    let artist = first_artist(group)?.get("artist")?;
    artist
        .get("country")
        .and_then(Value::as_str)
        .filter(|c| is_real_country(c))
        .map(String::from)
        .or_else(|| {
            artist
                .get("area")?
                .get("name")?
                .as_str()
                .filter(|n| !n.is_empty())
                .map(String::from)
        })
}

/// Takes the most-voted genres, capped so a heavily tagged album does not arrive with
/// thirty of them.
pub(super) fn collect_genres(value: Option<&Value>) -> Vec<String> {
    let Some(list) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut scored: Vec<(i64, String)> = list
        .iter()
        .filter_map(|g| {
            let name = g.get("name")?.as_str()?.to_string();
            let count = g.get("count").and_then(Value::as_i64).unwrap_or(0);
            Some((count, name))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().take(5).map(|(_, name)| name).collect()
}

/// Escapes the Lucene syntax characters MusicBrainz's search index understands.
///
/// The Java interpolated user text straight into the query, so a title containing a
/// quote or a colon silently produced a malformed search and no results. "Alien Lanes
/// (Deluxe)" or an album with a `:` in it would both have hit this.
pub(super) fn escape_lucene(input: &str) -> String {
    const SPECIAL: &[char] = &[
        '\\', '+', '-', '!', '(', ')', ':', '^', '[', ']', '"', '{', '}', '~', '*', '?', '|', '&',
        '/',
    ];
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if SPECIAL.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Percent-encodes a query-string value. Only a handful of characters matter here and
/// pulling in a URL crate for them is not worth the dependency.
pub(super) fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(score: i64, title: &str, primary: &str, secondary: &[&str], count: i64) -> Value {
        serde_json::json!({
            "id": title.to_lowercase().replace(' ', "-"),
            "score": score, "title": title, "primary-type": primary,
            "secondary-types": secondary, "count": count,
        })
    }

    fn picked(hits: &[Value]) -> String {
        pick_release_group(hits).unwrap()["id"].as_str().unwrap().to_string()
    }

    #[test]
    fn prefers_the_album_over_a_single_of_the_same_name() {
        // Live order for "The Weeknd - After Hours": the single first.
        let hits = [
            hit(100, "single", "Single", &[], 2),
            hit(100, "album", "Album", &[], 35),
            hit(93, "remixes", "EP", &["Remix"], 6),
        ];
        assert_eq!(picked(&hits), "album");
    }

    #[test]
    fn prefers_the_studio_album_over_live_and_compilation_albums() {
        // Live order for "Metallica - Metallica": the 1991 album fifth.
        let hits = [
            hit(100, "live", "Album", &["Live"], 1),
            hit(100, "interview", "Other", &["Interview"], 1),
            hit(100, "compilation", "Album", &["Compilation"], 1),
            hit(100, "album", "Album", &[], 59),
        ];
        assert_eq!(picked(&hits), "album");
    }

    #[test]
    fn the_most_pressed_album_wins_between_two_plain_ones() {
        let hits = [hit(100, "obscure", "Album", &[], 1), hit(100, "canonical", "Album", &[], 40)];
        assert_eq!(picked(&hits), "canonical");
    }

    #[test]
    fn a_better_text_match_is_never_overruled_by_type() {
        // The preference reorders ties. A search for an EP must still find the EP rather
        // than a looser-matching album that happens to have more pressings.
        let hits = [hit(100, "the ep", "EP", &[], 1), hit(87, "an album", "Album", &[], 50)];
        assert_eq!(picked(&hits), "the-ep");
    }

    #[test]
    fn keeps_musicbrainz_order_when_nothing_else_separates_hits() {
        let hits = [hit(100, "first", "Album", &[], 3), hit(100, "second", "Album", &[], 3)];
        assert_eq!(picked(&hits), "first");
    }

    #[test]
    fn an_empty_page_picks_nothing() {
        assert!(pick_release_group(&[]).is_none());
    }

    #[test]
    fn reads_the_album_from_a_release_group() {
        let group = serde_json::json!({
            "id": "180560ee-2d9d-33cf-8de7-cdaaba610739",
            "title": "City of Evil",
            "first-release-date": "2005-06-06",
            "artist-credit": [{"name": "Avenged Sevenfold",
                               "artist": {"name": "Avenged Sevenfold", "country": "US"}}],
        });
        let m = read_release_group(&group);
        assert_eq!(m.title.as_deref(), Some("City of Evil"));
        assert_eq!(m.artist.as_deref(), Some("Avenged Sevenfold"));
        assert_eq!(m.release_year, Some(2005));
        assert_eq!(m.musicbrainz_release_group_id.as_deref(), Some("180560ee-2d9d-33cf-8de7-cdaaba610739"));
        assert_eq!(artist_country(&group).as_deref(), Some("US"));
    }

    #[test]
    fn an_artist_without_a_country_code_falls_back_to_their_area() {
        let group = serde_json::json!({"artist-credit": [{"artist": {
            "name": "Radiohead", "country": "XE", "area": {"name": "England"}}}]});
        assert_eq!(artist_country(&group).as_deref(), Some("England"));

        let nothing = serde_json::json!({"artist-credit": [{"artist": {"name": "Anon"}}]});
        assert_eq!(artist_country(&nothing), None, "no guess from anywhere else");
    }

    #[test]
    fn a_separator_searches_artist_and_title_separately() {
        assert_eq!(
            search_query("Avenged Sevenfold - Nightmare").as_deref(),
            Some(r#"artist:"Avenged Sevenfold" AND releasegroup:"Nightmare""#)
        );
    }

    #[test]
    fn en_and_em_dashes_separate_too() {
        for text in ["Slint – Spiderland", "Slint — Spiderland"] {
            assert_eq!(split_artist_title(text), Some(("Slint", "Spiderland")), "{text}");
        }
    }

    #[test]
    fn a_dash_inside_a_name_is_not_a_separator() {
        // No spaces around it, or nothing on one side.
        assert_eq!(split_artist_title("Blink-182 Enema of the State"), None);
        assert_eq!(split_artist_title(" - Spiderland"), None);
    }

    #[test]
    fn words_without_a_separator_match_across_artist_and_title() {
        assert_eq!(
            search_query("avenged sevenfold nightmare").as_deref(),
            Some(concat!(
                r#"releasegroup:"avenged sevenfold nightmare"^3 OR ("#,
                r#"(artist:"avenged" OR releasegroup:"avenged") AND "#,
                r#"(artist:"sevenfold" OR releasegroup:"sevenfold") AND "#,
                r#"(artist:"nightmare" OR releasegroup:"nightmare"))"#
            ))
        );
    }

    #[test]
    fn query_syntax_in_typed_words_is_escaped() {
        let q = search_query("AC/DC \"Back\"").unwrap();
        assert!(q.contains(r#"artist:"AC\/DC""#), "{q}");
        assert!(q.contains(r#"artist:"\"Back\"""#), "{q}");
    }

    #[test]
    fn words_with_no_letters_are_not_required() {
        // A stray dash would otherwise have to appear in the artist or title, and nothing
        // ever would match.
        let q = search_query("duster & stratosphere").unwrap();
        assert!(!q.contains(r#"artist:"\&""#) && !q.contains(r#"artist:"&""#), "{q}");
        assert_eq!(search_query("  -  "), None);
        assert_eq!(search_query(""), None);
    }

    #[test]
    fn the_loose_fallback_is_just_the_words() {
        assert_eq!(loose_query("slint and spiderland").as_deref(), Some("slint and spiderland"));
        assert_eq!(loose_query("Slint – Spiderland 1991").as_deref(), Some("Slint Spiderland 1991"));
        assert_eq!(loose_query("–"), None);
    }

    #[test]
    fn ranks_every_hit_not_only_the_first() {
        // Live order for "avenged sevenfold nightmare": the single first.
        let hits = [
            hit(100, "single", "Single", &[], 3),
            hit(100, "album", "Album", &[], 15),
            hit(74, "soundfont", "Album", &[], 1),
            hit(74, "drum cover", "Single", &[], 2),
        ];
        let order: Vec<&str> = rank_release_groups(&hits)
            .iter()
            .map(|h| h["id"].as_str().unwrap())
            .collect();
        assert_eq!(order, ["album", "single", "soundfont", "drum-cover"]);
    }

    #[test]
    fn reads_a_candidate_for_the_list() {
        let group = serde_json::json!({
            "id": "abc", "title": "Metallica", "first-release-date": "1996",
            "primary-type": "Album", "secondary-types": ["Live"],
            "disambiguation": "  bootleg  ",
            "artist-credit": [{"name": "Metallica"}],
        });
        let c = read_candidate(&group).unwrap();
        assert_eq!(c.musicbrainz_release_group_id, "abc");
        assert_eq!(c.title.as_deref(), Some("Metallica"));
        assert_eq!(c.release_year, Some(1996));
        assert_eq!(c.primary_type.as_deref(), Some("Album"));
        assert_eq!(c.secondary_types, vec!["Live"]);
        assert_eq!(c.disambiguation.as_deref(), Some("bootleg"));
        assert_eq!(c.album_art_url, "https://coverartarchive.org/release-group/abc/front-250");
    }

    #[test]
    fn a_hit_without_an_id_is_not_a_candidate() {
        // It could not be looked up once chosen.
        assert!(read_candidate(&serde_json::json!({"title": "x"})).is_none());
        let blank = serde_json::json!({"id": "a", "disambiguation": ""});
        assert_eq!(read_candidate(&blank).unwrap().disambiguation, None);
    }

    #[test]
    fn escapes_lucene_syntax() {
        assert_eq!(escape_lucene("Spiderland"), "Spiderland");
        assert_eq!(escape_lucene("Alien Lanes (Deluxe)"), "Alien Lanes \\(Deluxe\\)");
        assert_eq!(escape_lucene(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_lucene("A: B"), "A\\: B");
        assert_eq!(escape_lucene("AC/DC"), "AC\\/DC");
        assert_eq!(escape_lucene("2 + 2"), "2 \\+ 2");
    }

    #[test]
    fn urlencodes_reserved_and_unicode() {
        assert_eq!(urlencode("abc-123_x.y~z"), "abc-123_x.y~z");
        assert_eq!(urlencode("a b"), "a%20b");
        assert_eq!(urlencode("a&b=c"), "a%26b%3Dc");
        assert_eq!(urlencode("Rós"), "R%C3%B3s", "utf-8 is encoded per byte");
    }

    #[test]
    fn genres_are_ranked_by_vote_then_name() {
        let json = serde_json::json!([
            {"name": "ambient", "count": 3},
            {"name": "jazz", "count": 9},
            {"name": "post-rock", "count": 9},
            {"name": "drone"}
        ]);
        assert_eq!(
            collect_genres(Some(&json)),
            vec!["jazz", "post-rock", "ambient", "drone"]
        );
    }

    #[test]
    fn genres_are_capped_and_tolerate_missing_data() {
        assert!(collect_genres(None).is_empty());
        assert!(collect_genres(Some(&serde_json::json!("not a list"))).is_empty());

        let many = serde_json::json!((0..12)
            .map(|i| serde_json::json!({"name": format!("g{i:02}"), "count": 100 - i}))
            .collect::<Vec<_>>());
        assert_eq!(collect_genres(Some(&many)).len(), 5);
    }

    #[test]
    fn retries_only_what_is_worth_retrying() {
        // Transient: the server is busy or we went too fast.
        for status in [429, 500, 502, 503, 504] {
            assert!(is_retryable(status), "{status} should be retried");
        }
        // Real answers. 404 above all: retrying it burns the one-per-second budget
        // three times over to learn the same thing.
        for status in [200, 400, 401, 403, 404, 301] {
            assert!(!is_retryable(status), "{status} must not be retried");
        }
    }

    #[test]
    fn filters_out_non_country_release_codes() {
        assert!(is_real_country("US"));
        assert!(is_real_country("GB"));
        assert!(is_real_country("JP"));
        // Correct answers to "where was it released", useless for "where is it from".
        assert!(!is_real_country("XW"), "worldwide");
        assert!(!is_real_country("XE"), "Europe");
        assert!(!is_real_country(""));
    }
}
