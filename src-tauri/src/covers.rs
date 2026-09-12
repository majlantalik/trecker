//! Album covers, kept on disk and served to the webview through the `cover:` protocol.
//!
//! The database stores each cover's remote URL, which stays the source of truth: it is what
//! an export carries and what a refresh replaces. The webview never loads that URL itself.
//! Every cover is requested as `cover://localhost/<url>`, and this module answers from the
//! cache directory, downloading on the first request.
//!
//! So a cover is fetched once, the first time anything shows it, and never again. After
//! that the library works offline and viewing it contacts nobody, which is the same reason
//! the fonts are bundled. The content security policy has no `https:` in `img-src`, so an
//! image tag that bypasses this module fails visibly instead of quietly going to the
//! network.
//!
//! Everything here is derived data. Deleting the directory loses nothing but the time to
//! download the covers again, which is why it lives in the OS cache directory and why the
//! Info view can clear it.

use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tauri::http::{header, Response, StatusCode};
use tokio::sync::Semaphore;

/// The URI scheme the webview uses. `frontend/src/api/cache.ts` builds URLs for it.
pub const SCHEME: &str = "cover";

/// A cover is a few tens of kilobytes. Anything this large is not a thumbnail, and refusing it
/// keeps a mistyped URL from filling the cache with someone's 40 MB scan.
const MAX_BYTES: u64 = 5 * 1024 * 1024;

const TIMEOUT: Duration = Duration::from_secs(15);

/// A library page shows twenty covers at once. Downloading them all in parallel is rude to
/// the archive and gains little, so a handful go at a time and the rest queue.
const PARALLEL_DOWNLOADS: usize = 4;

/// Written first, then renamed into place, so a crash mid-download never leaves a truncated
/// file that would be served as a cover.
const PARTIAL: &str = "part";

pub struct CoverCache {
    dir: PathBuf,
    http: reqwest::Client,
    downloads: Semaphore,
}

/// What the cache holds, or what a clear removed.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoverCacheInfo {
    pub path: String,
    pub count: u64,
    pub size_bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cover {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CoverError {
    /// Not a URL this module will fetch.
    BadUrl,
    /// Fetched, or tried to, and got no usable image.
    Unavailable,
}

impl CoverCache {
    pub fn new(dir: PathBuf, version: &str) -> Self {
        Self {
            dir,
            http: reqwest::Client::builder()
                .user_agent(format!("Trecker/{version} ( https://github.com/mtulek/trecker )"))
                .timeout(TIMEOUT)
                .build()
                .unwrap_or_default(),
            downloads: Semaphore::new(PARALLEL_DOWNLOADS),
        }
    }

    /// The cover for a stored URL, from disk if it is there and downloaded if it is not.
    pub async fn get(&self, url: &str) -> Result<Cover, CoverError> {
        if !url.starts_with("https://") {
            // Parity with the content security policy this module replaced, which allowed
            // images over https and nothing else. Without this, a URL in an imported file
            // could make the app fetch plain http or a local-network address on its behalf.
            return Err(CoverError::BadUrl);
        }

        if let Some(cover) = self.cached(url) {
            return Ok(cover);
        }

        let _permit = self.downloads.acquire().await.map_err(|_| CoverError::Unavailable)?;

        // Another request for the same cover may have finished while this one queued.
        if let Some(cover) = self.cached(url) {
            return Ok(cover);
        }

        for candidate in candidates(url) {
            if let Some(bytes) = self.download(&candidate).await {
                if let Some(content_type) = sniff(&bytes) {
                    // A cover that downloads but cannot be written is still worth showing.
                    let _ = self.store(url, &bytes);
                    return Ok(Cover { bytes, content_type });
                }
            }
        }
        Err(CoverError::Unavailable)
    }

    fn path_for(&self, url: &str) -> PathBuf {
        self.dir.join(key(url))
    }

    fn cached(&self, url: &str) -> Option<Cover> {
        let path = self.path_for(url);
        let bytes = std::fs::read(&path).ok()?;
        match sniff(&bytes) {
            Some(content_type) => Some(Cover { bytes, content_type }),
            None => {
                // Only images are ever written, so this is a damaged file. Drop it and let
                // the caller download a fresh copy.
                let _ = std::fs::remove_file(&path);
                None
            }
        }
    }

    fn store(&self, url: &str, bytes: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.path_for(url);
        let partial = path.with_extension(PARTIAL);
        std::fs::write(&partial, bytes)?;
        std::fs::rename(&partial, &path)
    }

    async fn download(&self, url: &str) -> Option<Vec<u8>> {
        let response = self.http.get(url).send().await.ok()?;
        if !response.status().is_success() {
            return None;
        }
        if response.content_length().is_some_and(|n| n > MAX_BYTES) {
            return None;
        }
        let bytes = response.bytes().await.ok()?;
        (bytes.len() as u64 <= MAX_BYTES).then(|| bytes.to_vec())
    }

    pub fn info(&self) -> AppResult<CoverCacheInfo> {
        let (count, size_bytes) = self.files()?.iter().fold((0, 0), |(c, s), (_, len)| (c + 1, s + len));
        Ok(CoverCacheInfo {
            path: self.dir.display().to_string(),
            count,
            size_bytes,
        })
    }

    /// Deletes every cached cover and reports what was removed. Covers download again the
    /// next time they are shown.
    pub fn clear(&self) -> AppResult<CoverCacheInfo> {
        let mut removed = CoverCacheInfo {
            path: self.dir.display().to_string(),
            count: 0,
            size_bytes: 0,
        };
        for (path, len) in self.files()? {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    removed.count += 1;
                    removed.size_bytes += len;
                }
                // Already gone, most likely removed by a concurrent clear. Nothing to report.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(AppError::Internal(format!(
                        "could not remove {}: {e}",
                        path.display()
                    )))
                }
            }
        }
        Ok(removed)
    }

    /// Every file in the cache directory, partial downloads included, so a clear leaves
    /// nothing behind. A directory that does not exist yet is an empty cache.
    fn files(&self) -> AppResult<Vec<(PathBuf, u64)>> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(AppError::Internal(format!(
                    "could not read {}: {e}",
                    self.dir.display()
                )))
            }
        };
        Ok(entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let meta = entry.metadata().ok()?;
                meta.is_file().then(|| (entry.path(), meta.len()))
            })
            .collect())
    }
}

/// Answers one `cover:` request from the webview.
pub async fn respond(cache: &CoverCache, request_path: &str) -> Response<Vec<u8>> {
    let result = match url_from_path(request_path) {
        Some(url) => cache.get(&url).await,
        None => Err(CoverError::BadUrl),
    };
    let builder = Response::builder();
    let response = match result {
        Ok(cover) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, cover.content_type)
            .body(cover.bytes),
        Err(CoverError::BadUrl) => builder.status(StatusCode::BAD_REQUEST).body(Vec::new()),
        Err(CoverError::Unavailable) => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.unwrap_or_default()
}

/// The remote URL a request is for. The webview sends it percent-encoded as the whole path,
/// because `convertFileSrc` runs it through `encodeURIComponent`.
fn url_from_path(path: &str) -> Option<String> {
    percent_decode(path.strip_prefix('/').unwrap_or(path))
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = input.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The URLs to try for a stored cover, smallest first.
///
/// Covers are shown at 260px at most, so the Cover Art Archive's 250px thumbnail is enough
/// and a third the size of the 500px one. The resolver already stores 250px URLs; this
/// catches the 500px and 1200px URLs stored before it did, or arriving in an imported file.
/// The stored URL is the fallback, in case the archive has not generated a small thumbnail.
fn candidates(url: &str) -> Vec<String> {
    match small_thumbnail(url) {
        Some(small) => vec![small, url.to_string()],
        None => vec![url.to_string()],
    }
}

/// The archive's documented thumbnail URLs end in `-250.jpg`, `-500.jpg` or `-1200.jpg`.
/// Only those are rewritten; a full-size original or any other host is left alone.
fn small_thumbnail(url: &str) -> Option<String> {
    if !url.starts_with("https://coverartarchive.org/") {
        return None;
    }
    ["-500.jpg", "-1200.jpg"]
        .iter()
        .find_map(|size| url.strip_suffix(size))
        .map(|stem| format!("{stem}-250.jpg"))
}

/// The file name for a cover: a hash of its stored URL.
///
/// FNV-1a rather than std's `DefaultHasher`, whose algorithm is allowed to change between
/// Rust releases and would silently orphan the whole cache after a toolchain upgrade. The
/// size is part of the input, so changing the preferred thumbnail later starts a fresh cache
/// instead of serving the old size forever.
fn key(url: &str) -> String {
    format!("{:016x}", fnv1a64(format!("250:{url}").as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// The image type, read from the first bytes rather than trusted from a server's headers.
/// Anything that is not an image, such as an HTML error page served with status 200, is
/// refused and never cached.
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("image/png"),
        [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10, b'J', b'F', b'I', b'F'];
    const CAA: &str = "https://coverartarchive.org/release/4f7c1a59-92b1-4ba7-919f-b61a3b4b8d2a/12051036941";

    fn cache() -> (tempfile::TempDir, CoverCache) {
        let dir = tempfile::tempdir().unwrap();
        let cache = CoverCache::new(dir.path().join("covers"), "0.1.0-test");
        (dir, cache)
    }

    #[test]
    fn fnv_matches_the_published_test_vectors() {
        // If this changes, every existing cache file becomes unreachable.
        assert_eq!(fnv1a64(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a64(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn a_key_is_a_stable_file_name_per_url() {
        let a = key("https://example.org/a.jpg");
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(a, key("https://example.org/a.jpg"));
        assert_ne!(a, key("https://example.org/b.jpg"));
    }

    #[test]
    fn archive_thumbnails_are_fetched_at_250px() {
        assert_eq!(small_thumbnail(&format!("{CAA}-500.jpg")), Some(format!("{CAA}-250.jpg")));
        assert_eq!(small_thumbnail(&format!("{CAA}-1200.jpg")), Some(format!("{CAA}-250.jpg")));
    }

    #[test]
    fn everything_else_is_fetched_as_stored() {
        // Already small, a full-size original with no thumbnail suffix, and another host.
        assert_eq!(small_thumbnail(&format!("{CAA}-250.jpg")), None);
        assert_eq!(small_thumbnail(&format!("{CAA}.jpg")), None);
        assert_eq!(small_thumbnail("https://example.org/cover-500.jpg"), None);
    }

    #[test]
    fn the_stored_url_is_the_fallback_after_the_small_thumbnail() {
        let stored = format!("{CAA}-500.jpg");
        assert_eq!(candidates(&stored), vec![format!("{CAA}-250.jpg"), stored.clone()]);
        assert_eq!(candidates("https://example.org/a.png"), vec!["https://example.org/a.png"]);
    }

    #[test]
    fn recognises_image_types_by_their_bytes() {
        assert_eq!(sniff(JPEG), Some("image/jpeg"));
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\n...."), Some("image/png"));
        assert_eq!(sniff(b"GIF89a..."), Some("image/gif"));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
    }

    #[test]
    fn refuses_anything_that_is_not_an_image() {
        // The case that matters: an error page served with a 200.
        assert_eq!(sniff(b"<!DOCTYPE html><html>"), None);
        assert_eq!(sniff(b""), None);
        assert_eq!(sniff(&[0xFF, 0xD8]), None, "too short to be a JPEG");
    }

    #[test]
    fn decodes_what_encode_uri_component_produces() {
        // encodeURIComponent("https://coverartarchive.org/release/x/1-250.jpg")
        let path = "/https%3A%2F%2Fcoverartarchive.org%2Frelease%2Fx%2F1-250.jpg";
        assert_eq!(
            url_from_path(path).as_deref(),
            Some("https://coverartarchive.org/release/x/1-250.jpg")
        );
        assert_eq!(percent_decode("R%C3%B3s").as_deref(), Some("Rós"), "utf-8 across bytes");
    }

    #[test]
    fn refuses_malformed_percent_encoding() {
        assert_eq!(percent_decode("bad%2"), None);
        assert_eq!(percent_decode("bad%zz"), None);
        assert_eq!(percent_decode("%FF"), None, "not utf-8");
    }

    #[tokio::test]
    async fn only_https_urls_are_fetched() {
        let (_d, cache) = cache();
        for url in ["http://coverartarchive.org/a.jpg", "file:///etc/passwd", "ftp://x/y", ""] {
            assert_eq!(cache.get(url).await, Err(CoverError::BadUrl), "{url}");
        }
    }

    #[tokio::test]
    async fn a_stored_cover_is_served_without_the_network() {
        // The URL does not resolve; a network request would fail. Serving it proves the
        // cache answered.
        let (_d, cache) = cache();
        let url = "https://cover.invalid/a-500.jpg";
        cache.store(url, JPEG).unwrap();
        let cover = cache.get(url).await.unwrap();
        assert_eq!(cover.bytes, JPEG);
        assert_eq!(cover.content_type, "image/jpeg");
    }

    #[test]
    fn a_damaged_cache_file_is_discarded_rather_than_served() {
        let (_d, cache) = cache();
        let url = "https://cover.invalid/a.jpg";
        cache.store(url, b"<html>not an image").unwrap();
        assert_eq!(cache.cached(url), None, "a miss, so the caller downloads again");
        assert!(!cache.path_for(url).exists(), "the bad file is removed");
    }

    #[tokio::test]
    async fn the_protocol_answers_with_status_and_type() {
        let (_d, cache) = cache();
        let url = "https://cover.invalid/a.jpg";
        cache.store(url, JPEG).unwrap();

        let ok = respond(&cache, "/https%3A%2F%2Fcover.invalid%2Fa.jpg").await;
        assert_eq!(ok.status(), StatusCode::OK);
        assert_eq!(ok.headers()[header::CONTENT_TYPE], "image/jpeg");
        assert_eq!(ok.body(), JPEG);

        let bad = respond(&cache, "/http%3A%2F%2Fcover.invalid%2Fa.jpg").await;
        assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

        let garbled = respond(&cache, "/%zz").await;
        assert_eq!(garbled.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn an_empty_or_missing_cache_reports_nothing() {
        let (_d, cache) = cache();
        let info = cache.info().unwrap();
        assert_eq!((info.count, info.size_bytes), (0, 0), "directory not created yet");
        assert_eq!(cache.clear().unwrap().count, 0);
    }

    #[test]
    fn info_counts_files_and_clear_removes_all_of_them() {
        let (_d, cache) = cache();
        cache.store("https://x.invalid/1.jpg", JPEG).unwrap();
        cache.store("https://x.invalid/2.jpg", JPEG).unwrap();
        // A download interrupted by a crash leaves a partial file. It still takes space, so
        // it is counted and cleared.
        std::fs::write(cache.dir.join("deadbeef.part"), b"half").unwrap();

        let info = cache.info().unwrap();
        assert_eq!(info.count, 3);
        assert_eq!(info.size_bytes, 2 * JPEG.len() as u64 + 4);

        let removed = cache.clear().unwrap();
        assert_eq!(removed, info, "reports exactly what it removed");
        assert_eq!(cache.info().unwrap().count, 0);
        assert!(cache.dir.exists(), "the directory itself stays");
    }

    /// Downloads a real cover. City of Evil's stored URL is the 500px thumbnail; the cache
    /// has to store the 250px one.
    #[tokio::test]
    #[ignore = "makes real network requests"]
    async fn downloads_the_250px_thumbnail_for_a_stored_500px_url() {
        let (_d, cache) = cache();
        let url = format!("{CAA}-500.jpg");

        let cover = cache.get(&url).await.expect("the cover should download");
        assert_eq!(cover.content_type, "image/jpeg");
        assert_eq!(jpeg_width(&cover.bytes), Some(250));
        assert_eq!(std::fs::read(cache.path_for(&url)).unwrap(), cover.bytes, "and it is cached");
    }

    /// Reads a baseline or progressive JPEG's width from its start-of-frame marker.
    fn jpeg_width(bytes: &[u8]) -> Option<u16> {
        let mut i = 2;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            if matches!(marker, 0xC0 | 0xC1 | 0xC2) {
                return Some(u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]));
            }
            i += 2 + len;
        }
        None
    }
}
