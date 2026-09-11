//! MusicBrainz client. Port of `MusicBrainzService`.
//!
//! MusicBrainz is the only provider that needs no account, no key and no approval, which
//! is why it is the backbone rather than an enrichment. In exchange it asks for two
//! things and enforces both: a descriptive `User-Agent`, and no more than one request per
//! second per client.

use super::{Resolver, TIMEOUT};
use crate::domain::ResolvedMetadata;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const API: &str = "https://musicbrainz.org/ws/2";

#[derive(Debug, Default)]
pub struct ReleaseDetails {
    pub genres: Vec<String>,
    pub first_release_year: Option<i32>,
}

/// MusicBrainz country codes that are not countries.
///
/// `XW` is "worldwide" and `XE` is "Europe". Both are correct answers to "where was this
/// released" and useless answers to "where is this music from", which is what a By
/// Country chart is actually asking. Treated as absent so the artist lookup can fill in.
fn is_real_country(code: &str) -> bool {
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

    async fn mb_get(&self, url: &str) -> Option<Value> {
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

    /// Searches for a release. `Artist - Title` is split into a two-field Lucene query,
    /// anything else is treated as a release title.
    pub async fn mb_search_release(&self, query: &str) -> Option<ResolvedMetadata> {
        let lucene = match query.split_once(" - ") {
            Some((artist, release)) => format!(
                "artist:\"{}\" AND release:\"{}\"",
                escape_lucene(artist.trim()),
                escape_lucene(release.trim())
            ),
            None => format!("release:\"{}\"", escape_lucene(query.trim())),
        };

        let url = format!(
            "{API}/release?query={}&fmt=json&limit=1",
            urlencode(&lucene)
        );
        let body = self.mb_get(&url).await?;
        let first = body.get("releases")?.as_array()?.first()?;

        let mbid = first.get("id")?.as_str()?.to_string();
        let title = first.get("title").and_then(Value::as_str).map(String::from);
        let country = first
            .get("country")
            .and_then(Value::as_str)
            .filter(|c| is_real_country(c))
            .map(String::from);
        let release_year = first
            .get("date")
            .and_then(Value::as_str)
            .and_then(|d| d.get(0..4))
            .and_then(|y| y.parse().ok());

        // The artist name sits under artist-credit, which has two shapes depending on
        // whether the release is credited to a single artist or a collaboration.
        let artist = first
            .get("artist-credit")
            .and_then(Value::as_array)
            .and_then(|credits| credits.first())
            .and_then(|c| {
                c.get("name")
                    .and_then(Value::as_str)
                    .or_else(|| c.get("artist")?.get("name")?.as_str())
            })
            .map(String::from);

        if artist.is_none() && title.is_none() {
            return None;
        }

        Some(ResolvedMetadata {
            artist,
            title,
            release_year,
            album_art_url: None,
            country,
            streaming_links: HashMap::new(),
            genres: Vec::new(),
            spotify_id: None,
            musicbrainz_id: Some(mbid),
        })
    }

    /// Best-effort second lookup, for the two things the search result cannot give us.
    ///
    /// Genres, because the web app got those from Spotify and they would otherwise be
    /// lost along with the By Genre chart. And the original release year, because a
    /// search returns one *pressing*: asking for "Slint - Spiderland" can land on a 2014
    /// reissue and report 2014, which is not what anyone means by an album's year. The
    /// release group carries `first-release-date`, which is the date of the album itself.
    ///
    /// Costs one rate-limited request, so it is separate and allowed to fail.
    pub async fn mb_release_details(&self, mbid: &str) -> ReleaseDetails {
        let url = format!("{API}/release/{mbid}?inc=genres+release-groups&fmt=json");
        let Some(body) = self.mb_get(&url).await else {
            return ReleaseDetails::default();
        };

        let group = body.get("release-group");

        // Genres live on the release when someone has tagged that specific pressing, and
        // otherwise on the release group, which is the more commonly tagged of the two.
        let mut genres = collect_genres(body.get("genres"));
        if genres.is_empty() {
            genres = collect_genres(group.and_then(|rg| rg.get("genres")));
        }

        let first_release_year = group
            .and_then(|rg| rg.get("first-release-date"))
            .and_then(Value::as_str)
            .and_then(|d| d.get(0..4))
            .and_then(|y| y.parse().ok());

        ReleaseDetails {
            genres,
            first_release_year,
        }
    }

    /// Country of an artist, falling back to the broader area when the country is unset.
    pub async fn mb_artist_country(&self, artist: &str) -> Option<String> {
        let url = format!(
            "{API}/artist?query={}&fmt=json&limit=1",
            urlencode(&escape_lucene(artist))
        );
        let body = self.mb_get(&url).await?;
        let first = body.get("artists")?.as_array()?.first()?;

        first
            .get("country")
            .and_then(Value::as_str)
            .filter(|c| !c.is_empty())
            .map(String::from)
            .or_else(|| {
                first
                    .get("area")?
                    .get("name")?
                    .as_str()
                    .filter(|n| !n.is_empty())
                    .map(String::from)
            })
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

/// Takes the most-voted genres, capped so a heavily tagged release does not arrive with
/// thirty of them.
fn collect_genres(value: Option<&Value>) -> Vec<String> {
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
fn escape_lucene(input: &str) -> String {
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
fn urlencode(input: &str) -> String {
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
