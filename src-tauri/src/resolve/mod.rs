//! Metadata resolution. Port of `MetadataResolverService`.
//!
//! The web app fanned out to Spotify, Tidal, YouTube and MusicBrainz in parallel and
//! merged the results, with MusicBrainz authoritative for identity and Spotify supplying
//! art and genres. Only MusicBrainz survives the move to a native app, because it is the
//! only one of the four that needs no credential:
//!
//!   Spotify  needs a client secret (or per-user OAuth against an app capped at five
//!            allowlisted users whose owner must hold a Premium subscription)
//!   Tidal    needs a client secret, via the same client-credentials flow
//!   YouTube  needs an API key, which anyone can pull out of a shipped binary and spend
//!
//! A secret in a distributed binary is not a secret. So the shape changes: MusicBrainz
//! for identity, the Cover Art Archive for artwork, MusicBrainz genres in place of
//! Spotify's, and a pasted streaming URL is kept as a link rather than resolved.

pub mod coverart;
pub mod musicbrainz;

use crate::domain::{ResolveRequest, ResolvedMetadata};
use crate::error::{AppError, AppResult};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Per-request budget. Matches the 8 seconds the Java allowed.
pub const TIMEOUT: Duration = Duration::from_secs(8);

/// Whole-resolve budget. Larger than TIMEOUT because a resolve makes up to three
/// sequential calls and the MusicBrainz gate puts a second between two of them.
const TOTAL_BUDGET: Duration = Duration::from_secs(20);

pub struct Resolver {
    http: reqwest::Client,
    user_agent: String,
    /// When the last MusicBrainz request went out. See `mb_gate`.
    ///
    /// The limit is per client, so this only honours it if there is one Resolver in the
    /// process. There is: Tauri manages a single instance as shared state.
    mb_last_request: Mutex<Option<Instant>>,
}

impl Resolver {
    pub fn new(version: &str) -> Self {
        // MusicBrainz requires a User-Agent that identifies the application and offers a
        // way to make contact. Sending a generic one gets you blocked, correctly.
        let user_agent = format!("Trecker/{version} ( https://github.com/mtulek/trecker )");
        Self {
            http: reqwest::Client::builder()
                .user_agent(user_agent.clone())
                .build()
                .unwrap_or_default(),
            user_agent,
            mb_last_request: Mutex::new(None),
        }
    }

    pub async fn resolve(&self, request: ResolveRequest) -> AppResult<ResolvedMetadata> {
        let raw = request
            .url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| request.query.filter(|s| !s.trim().is_empty()))
            .ok_or_else(|| AppError::Invalid("nothing to resolve".into()))?;
        let raw = raw.trim().to_string();

        tokio::time::timeout(TOTAL_BUDGET, self.resolve_inner(raw))
            .await
            .map_err(|_| AppError::Resolve("lookup timed out".into()))?
    }

    async fn resolve_inner(&self, raw: String) -> AppResult<ResolvedMetadata> {
        // A pasted streaming link cannot be resolved without that service's credentials,
        // but it is still worth keeping: the link is what the user actually wanted to
        // save. So record it and search on whatever text we can recover from the URL.
        if let Some((service, url)) = detect_streaming_link(&raw) {
            let mut result = match slug_from_url(&raw) {
                Some(text) => self.search(&text).await.unwrap_or_default(),
                None => ResolvedMetadata::default(),
            };
            result.streaming_links.insert(service.to_string(), url);
            return Ok(result);
        }

        self.search(&raw)
            .await
            .ok_or_else(|| AppError::Resolve(format!("nothing found for '{raw}'")))
    }

    /// MusicBrainz for identity, then the Cover Art Archive and genres to fill it out.
    /// Both enrichment steps are best-effort: a release with no artwork and no tags is
    /// still a perfectly good result.
    async fn search(&self, query: &str) -> Option<ResolvedMetadata> {
        let mut result = self.mb_search_release(query).await?;

        if let Some(mbid) = result.musicbrainz_id.clone() {
            // Details first, because everything else depends on what it returns. The
            // search matched one pressing; this is what turns that into the album.
            let details = self.mb_release_details(&mbid).await;

            result.genres = details.genres;

            // The album's first release date beats the matched pressing's own date. A
            // search for Spiderland can land on the 2014 reissue, and 1991 is the answer
            // anyone actually wants.
            if let Some(year) = details.first_release_year {
                result.release_year = Some(year);
            }

            // The artist's country beats the pressing's. Where a particular edition was
            // sold is close to arbitrary, and "Avenged Sevenfold are Canadian" is simply
            // wrong. Falls back to the release country when the artist has none.
            if details.artist_country.is_some() {
                result.country = details.artist_country;
            }

            result.album_art_url = self
                .cover_art(&mbid, details.release_group_id.as_deref())
                .await;
        }

        Some(result)
    }
}

/// Recognises the services whose links are worth keeping even though their APIs are out
/// of reach. The URL is returned normalised of tracking parameters.
fn detect_streaming_link(input: &str) -> Option<(&'static str, String)> {
    let lower = input.to_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return None;
    }

    let service = if lower.contains("spotify.com") {
        "spotify"
    } else if lower.contains("tidal.com") {
        "tidal"
    } else if lower.contains("youtube.com") || lower.contains("youtu.be") {
        "youtube"
    } else if lower.contains("bandcamp.com") {
        "bandcamp"
    } else if lower.contains("music.apple.com") {
        "apple"
    } else {
        return None;
    };

    // Strip the query string: share links carry si= and utm_ parameters that identify the
    // person who shared them, and none of it belongs in a saved library.
    let cleaned = input.split(['?', '#']).next().unwrap_or(input).to_string();
    Some((service, cleaned))
}

/// Recovers searchable words from a URL path.
///
/// Bandcamp and Apple Music put the artist and album in the URL, so a paste can still be
/// resolved. Spotify and Tidal use opaque ids, which yield nothing searchable, and that
/// is the correct outcome: better an empty form with the link saved than a confident
/// match on a random hex string.
fn slug_from_url(url: &str) -> Option<String> {
    let after_scheme = url.split("://").nth(1)?;
    let mut host_and_path = after_scheme.splitn(2, '/');
    let host = host_and_path.next()?.to_lowercase();
    let path = host_and_path.next()?;
    let path = path.split(['?', '#']).next().unwrap_or(path);

    let words: Vec<String> = if host.contains("bandcamp.com") {
        // artist.bandcamp.com/album/the-white-birch
        let artist = host.split('.').next().unwrap_or_default().to_string();
        let album = path.rsplit('/').next().unwrap_or_default();
        vec![deslug(&artist), deslug(album)]
    } else if host.contains("music.apple.com") {
        // music.apple.com/us/album/the-white-birch/123456
        let mut segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        segments.retain(|s| !s.chars().all(|c| c.is_ascii_digit()));
        match segments.iter().position(|s| *s == "album") {
            Some(i) if i + 1 < segments.len() => vec![deslug(segments[i + 1])],
            _ => vec![],
        }
    } else {
        vec![]
    };

    let joined = words
        .into_iter()
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!joined.trim().is_empty()).then_some(joined)
}

fn deslug(s: &str) -> String {
    s.replace(['-', '_'], " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_services_worth_keeping_links_for() {
        for (url, expected) in [
            ("https://open.spotify.com/album/abc123", "spotify"),
            ("https://tidal.com/browse/album/12345", "tidal"),
            ("https://www.youtube.com/watch?v=xyz", "youtube"),
            ("https://youtu.be/xyz", "youtube"),
            ("https://duster.bandcamp.com/album/stratosphere", "bandcamp"),
            ("https://music.apple.com/us/album/scenery/1", "apple"),
        ] {
            assert_eq!(detect_streaming_link(url).unwrap().0, expected, "{url}");
        }
    }

    #[test]
    fn ignores_plain_text_and_unknown_hosts() {
        assert!(detect_streaming_link("Slint - Spiderland").is_none());
        assert!(detect_streaming_link("https://example.com/album/x").is_none());
        // No scheme means it is a search, not a link, even if it names a service.
        assert!(detect_streaming_link("spotify.com/album/x").is_none());
    }

    #[test]
    fn strips_sharing_and_tracking_parameters() {
        let (_, url) = detect_streaming_link("https://open.spotify.com/album/abc?si=trackingid").unwrap();
        assert_eq!(url, "https://open.spotify.com/album/abc");

        let (_, url) = detect_streaming_link("https://youtu.be/xyz?t=42&utm_source=x").unwrap();
        assert_eq!(url, "https://youtu.be/xyz");
    }

    #[test]
    fn recovers_search_text_from_descriptive_urls() {
        assert_eq!(
            slug_from_url("https://duster.bandcamp.com/album/stratosphere").as_deref(),
            Some("duster stratosphere")
        );
        assert_eq!(
            slug_from_url("https://music.apple.com/us/album/the-white-birch/1553176").as_deref(),
            Some("the white birch")
        );
    }

    #[test]
    fn recovers_nothing_from_opaque_ids() {
        // Better an empty form with the link saved than a confident match on a hex string.
        assert_eq!(slug_from_url("https://open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy"), None);
        assert_eq!(slug_from_url("https://tidal.com/browse/album/12345"), None);
        assert_eq!(slug_from_url("https://www.youtube.com/watch?v=abc"), None);
        assert_eq!(slug_from_url("not a url"), None);
    }

    /// Hits the live MusicBrainz and Cover Art Archive services, so it is excluded from
    /// normal runs. Use it to check the client still matches their API after an upgrade:
    ///
    ///     cargo test --lib -- --ignored --nocapture --test-threads=1
    ///
    /// `--test-threads=1` is not optional. The rate limiter lives on the Resolver, and
    /// each test builds its own, so running them in parallel triples the request rate and
    /// MusicBrainz answers 503. The app is safe because Tauri holds exactly one Resolver.
    #[tokio::test]
    #[ignore = "makes real network requests"]
    async fn resolves_a_known_release_against_the_live_services() {
        let r = Resolver::new("0.1.0-test");

        let found = r
            .resolve(ResolveRequest {
                query: Some("Slint - Spiderland".into()),
                url: None,
            })
            .await
            .expect("Spiderland should resolve");

        println!("{found:#?}");
        assert_eq!(found.artist.as_deref(), Some("Slint"));
        assert_eq!(found.title.as_deref(), Some("Spiderland"));
        assert_eq!(found.release_year, Some(1991));
        assert!(found.musicbrainz_id.is_some(), "identity comes from MusicBrainz");
        assert!(found.album_art_url.is_some(), "artwork comes from the Cover Art Archive");
        assert!(found.country.is_some());
    }

    #[tokio::test]
    #[ignore = "makes real network requests"]
    async fn rate_limit_gate_spaces_requests_out() {
        let r = Resolver::new("0.1.0-test");
        let start = std::time::Instant::now();
        let _ = r.mb_search_release("Slint - Spiderland").await;
        let _ = r.mb_search_release("Duster - Stratosphere").await;
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(1100),
            "two MusicBrainz calls must be at least 1.1s apart, took {elapsed:?}"
        );
    }

    #[tokio::test]
    #[ignore = "makes real network requests"]
    async fn a_pasted_spotify_link_is_kept_even_though_it_cannot_be_resolved() {
        let r = Resolver::new("0.1.0-test");
        let found = r
            .resolve(ResolveRequest {
                url: Some("https://open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy?si=abc".into()),
                query: None,
            })
            .await
            .expect("a link with no metadata is still a result");

        assert_eq!(
            found.streaming_links.get("spotify").map(String::as_str),
            Some("https://open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy"),
            "link saved, tracking parameter stripped"
        );
        assert!(found.artist.is_none(), "no credentials, so no metadata");
    }

    /// The release that failed in real use: the search matches a Canadian pressing with
    /// no cover art of its own, credited to a band from California.
    #[tokio::test]
    #[ignore = "makes real network requests"]
    async fn falls_back_to_the_album_when_the_pressing_has_no_art() {
        let r = Resolver::new("0.1.0-test");
        let found = r
            .resolve(ResolveRequest {
                query: Some("avenged sevenfold - city of evil".into()),
                url: None,
            })
            .await
            .expect("should resolve");

        println!("{found:#?}");
        assert_eq!(found.title.as_deref(), Some("City of Evil"));
        assert_eq!(found.release_year, Some(2005));
        assert_eq!(found.country.as_deref(), Some("US"), "the band, not the pressing");
        assert!(
            found.album_art_url.is_some(),
            "the pressing has no art but the release group does"
        );
        let art = found.album_art_url.unwrap();
        assert!(art.starts_with("https://"), "must be https or the CSP blocks it: {art}");
    }
}
