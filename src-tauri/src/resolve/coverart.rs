//! Cover Art Archive client. Port of `MusicBrainzService.fetchCoverArt`.
//!
//! A separate service from the MusicBrainz API, so it is not behind the one-per-second
//! gate. It is keyed by MusicBrainz release id, which is the whole reason the release
//! lookup has to happen first.

use super::{Resolver, TIMEOUT};
use serde_json::Value;
use std::time::Duration;

impl Resolver {
    /// Artwork for a specific pressing, falling back to the album.
    ///
    /// The pressing a MusicBrainz search happens to match often has no art uploaded even
    /// when the album plainly does: "Avenged Sevenfold - City of Evil" matches a Canadian
    /// edition with nothing in the archive, while the release group has the cover
    /// everyone would recognise. So a miss on the release is not a miss.
    pub async fn cover_art(&self, mbid: &str, release_group_id: Option<&str>) -> Option<String> {
        if let Some(url) = self.cover_art_for("release", mbid).await {
            return Some(url);
        }
        self.cover_art_for("release-group", release_group_id?).await
    }

    async fn cover_art_for(&self, kind: &str, id: &str) -> Option<String> {
        let url = format!("https://coverartarchive.org/{kind}/{id}");

        let mut attempt = 0;
        let body: Value = loop {
            let result = self
                .http
                .get(&url)
                .header("User-Agent", &self.user_agent)
                .header("Accept", "application/json")
                .timeout(TIMEOUT)
                .send()
                .await;

            match result {
                Ok(r) if r.status().is_success() => match r.json::<Value>().await {
                    Ok(v) => break v,
                    Err(_) => return None,
                },
                // A 404 is the normal answer for a release nobody has uploaded art for,
                // so it must not be retried; a 503 is the archive being busy, so it must.
                Ok(r) if super::musicbrainz::is_retryable(r.status().as_u16()) && attempt < 2 => {
                    attempt += 1;
                    let wait = super::musicbrainz::header_seconds(&r)
                        .unwrap_or_else(|| Duration::from_millis(500 * 2u64.pow(attempt)))
                        .min(Duration::from_secs(5));
                    tokio::time::sleep(wait).await;
                }
                Ok(_) => return None,
                Err(e) if attempt < 2 && (e.is_connect() || e.is_request()) => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
                }
                Err(_) => return None,
            }
        };

        pick_image(&body)
    }
}

/// Prefers the image flagged as the front cover, then falls back to the first one.
/// Thumbnails are tried 500 before 250 before 1200: 500 is the size the card grid
/// actually displays, and pulling 1200 for a 160px card wastes the user's bandwidth.
fn pick_image(body: &Value) -> Option<String> {
    let images = body.get("images")?.as_array()?;
    if images.is_empty() {
        return None;
    }

    let chosen = images
        .iter()
        .find(|img| img.get("front").and_then(Value::as_bool) == Some(true))
        .or_else(|| images.first())?;

    if let Some(thumbnails) = chosen.get("thumbnails") {
        for size in ["500", "250", "1200"] {
            if let Some(url) = thumbnails.get(size).and_then(Value::as_str) {
                return Some(https(url));
            }
        }
    }
    chosen.get("image").and_then(Value::as_str).map(https)
}

/// The Cover Art Archive hands out `http://` URLs in its JSON even though the same paths
/// serve fine over TLS. Stored as-is they would be blocked outright by the app's
/// content security policy, and every cover would silently fail to load. They are also
/// saved to the database, so the scheme is fixed here rather than at render time.
fn https(url: &str) -> String {
    match url.strip_prefix("http://") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefers_the_front_cover() {
        let body = json!({"images": [
            {"front": false, "thumbnails": {"500": "back-500"}},
            {"front": true,  "thumbnails": {"500": "front-500"}}
        ]});
        assert_eq!(pick_image(&body).as_deref(), Some("front-500"));
    }

    #[test]
    fn falls_back_to_the_first_image_when_none_is_flagged_front() {
        let body = json!({"images": [{"thumbnails": {"500": "first-500"}}, {"thumbnails": {"500": "second"}}]});
        assert_eq!(pick_image(&body).as_deref(), Some("first-500"));
    }

    #[test]
    fn prefers_500_then_250_then_1200() {
        let all = json!({"images": [{"front": true, "thumbnails": {"250": "a", "500": "b", "1200": "c"}}]});
        assert_eq!(pick_image(&all).as_deref(), Some("b"));

        let no_500 = json!({"images": [{"front": true, "thumbnails": {"250": "a", "1200": "c"}}]});
        assert_eq!(pick_image(&no_500).as_deref(), Some("a"));

        let only_large = json!({"images": [{"front": true, "thumbnails": {"1200": "c"}}]});
        assert_eq!(pick_image(&only_large).as_deref(), Some("c"));
    }

    #[test]
    fn falls_back_to_the_full_image_when_there_are_no_thumbnails() {
        let body = json!({"images": [{"front": true, "image": "full.jpg"}]});
        assert_eq!(pick_image(&body).as_deref(), Some("full.jpg"));
    }

    #[test]
    fn returns_nothing_for_empty_or_malformed_responses() {
        assert_eq!(pick_image(&json!({})), None);
        assert_eq!(pick_image(&json!({"images": []})), None);
        assert_eq!(pick_image(&json!({"images": "nope"})), None);
        assert_eq!(pick_image(&json!({"images": [{"front": true}]})), None);
    }

    #[test]
    fn upgrades_plaintext_urls_to_tls() {
        // The archive serves http:// in its JSON; the content security policy allows
        // https: only, so an unfixed URL means no cover art at all.
        let body = json!({"images": [{"front": true, "thumbnails": {"500":
            "http://coverartarchive.org/release/abc/123-500.jpg"}}]});
        assert_eq!(
            pick_image(&body).as_deref(),
            Some("https://coverartarchive.org/release/abc/123-500.jpg")
        );

        let full = json!({"images": [{"front": true, "image": "http://coverartarchive.org/x.jpg"}]});
        assert_eq!(pick_image(&full).as_deref(), Some("https://coverartarchive.org/x.jpg"));
    }

    #[test]
    fn leaves_urls_that_are_already_secure_alone() {
        assert_eq!(https("https://example.org/a.jpg"), "https://example.org/a.jpg");
        // Only the scheme prefix is rewritten, never the path.
        assert_eq!(https("https://x.org/http://y"), "https://x.org/http://y");
    }
}
