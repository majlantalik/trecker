//! Wire types shared with the frontend.
//!
//! Every struct here must serialize to exactly the interface of the same name in
//! `frontend/src/types/index.ts`. That file stays the single source of truth; these are
//! its Rust mirror. Phase 3 will back them with SQLite rows rather than in-memory state,
//! but the shapes must not change, because the Vue components consume them directly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseStatus {
    #[serde(rename = "QUEUED")]
    Queued,
    #[serde(rename = "LISTENED")]
    Listened,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub artist: String,
    pub title: String,
    pub release_year: Option<i32>,
    pub album_art_url: Option<String>,
    pub status: ReleaseStatus,
    pub discovery_link: Option<String>,
    pub streaming_links: HashMap<String, String>,
    pub country: Option<String>,
    pub rating: Option<f64>,
    pub did_not_finish: bool,
    pub date_listened: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub genres: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseRequest {
    pub artist: String,
    pub title: String,
    pub release_year: Option<i32>,
    pub album_art_url: Option<String>,
    pub country: Option<String>,
    pub discovery_link: Option<String>,
    pub streaming_links: Option<HashMap<String, String>>,
    pub genres: Option<Vec<String>>,
    // Part of the wire contract already, but only read once the catalog exists: these are
    // the dedup keys `ReleaseService.create()` looks up before inserting. Phase 3.
    #[allow(dead_code)]
    pub spotify_id: Option<String>,
    #[allow(dead_code)]
    pub musicbrainz_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUpdateRequest {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub album_art_url: Option<String>,
    pub status: Option<ReleaseStatus>,
    pub discovery_link: Option<String>,
    pub streaming_links: Option<HashMap<String, String>>,
    pub country: Option<String>,
    pub rating: Option<f64>,
    pub did_not_finish: Option<bool>,
    pub date_listened: Option<String>,
    pub notes: Option<String>,
    pub genres: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequest {
    pub url: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedMetadata {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub album_art_url: Option<String>,
    pub country: Option<String>,
    pub streaming_links: HashMap<String, String>,
    pub genres: Vec<String>,
    pub spotify_id: Option<String>,
    pub musicbrainz_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseFilterParams {
    pub status: Option<ReleaseStatus>,
    pub genre: Option<String>,
    pub country: Option<String>,
    pub rating_min: Option<f64>,
    pub rating_max: Option<f64>,
    pub year: Option<i32>,
    pub did_not_finish: Option<bool>,
    pub search: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub content: Vec<T>,
    pub total_elements: u64,
    pub total_pages: u32,
    pub number: u32,
    pub size: u32,
    pub first: bool,
    pub last: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDataPoint {
    pub year: i32,
    pub month: u32,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItem {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndEntry {
    pub rank: u32,
    pub release: Release,
}
