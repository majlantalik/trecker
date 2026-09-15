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
    /// The dedup key: a catalog row is found by this before a new one is inserted. A
    /// release group, so every pressing of one album resolves to the same row.
    pub musicbrainz_release_group_id: Option<String>,
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
    pub musicbrainz_release_group_id: Option<String>,
}

/// One possible match for a quick add search, shown in a list to choose from.
///
/// Deliberately thin: everything here comes from a single search request. Genres, country
/// and the chosen cover need a lookup each, which at one MusicBrainz request per second
/// would make a list of ten take ten seconds, so they are fetched only for the album the
/// person picks.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCandidate {
    pub musicbrainz_release_group_id: String,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    /// `Album`, `EP`, `Single`, `Broadcast` or `Other`, as MusicBrainz names them.
    pub primary_type: Option<String>,
    /// Such as `Live`, `Compilation` or `Remix`. Empty for a plain studio release.
    pub secondary_types: Vec<String>,
    /// MusicBrainz's own note telling same-named albums apart, when it has one.
    pub disambiguation: Option<String>,
    /// A small cover for the list. Not checked: many albums have none, and the image
    /// simply fails to load.
    pub album_art_url: String,
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

// ---------------------------------------------------------------- artists

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtistStatus {
    #[serde(rename = "TO_CHECK")]
    ToCheck,
    #[serde(rename = "CHECKED")]
    Checked,
}

impl ArtistStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtistStatus::ToCheck => "TO_CHECK",
            ArtistStatus::Checked => "CHECKED",
        }
    }
}

/// What you made of an artist once checked. Optional: checking someone out does not
/// require an opinion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtistVerdict {
    #[serde(rename = "LIKED")]
    Liked,
    #[serde(rename = "NOT_FOR_ME")]
    NotForMe,
}

impl ArtistVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtistVerdict::Liked => "LIKED",
            ArtistVerdict::NotForMe => "NOT_FOR_ME",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistLink {
    /// MusicBrainz's relationship type, such as `official homepage` or `bandcamp`.
    pub kind: String,
    pub url: String,
}

/// An artist on your list. `id` is the catalog artist id, as a release's is.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub musicbrainz_artist_id: Option<String>,
    pub name: String,
    pub disambiguation: Option<String>,
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub begin_year: Option<i32>,
    pub end_year: Option<i32>,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub links: Vec<ArtistLink>,
    pub status: ArtistStatus,
    pub verdict: Option<ArtistVerdict>,
    pub note: Option<String>,
    pub checked_at: Option<String>,
    /// When you added them, from the tracking row.
    pub created_at: String,
}

/// One possible match for an artist search. Everything here comes from the search request.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtistCandidate {
    pub musicbrainz_artist_id: String,
    pub name: String,
    pub disambiguation: Option<String>,
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub begin_year: Option<i32>,
    pub end_year: Option<i32>,
    /// Search results carry tags, not genres. Good enough to tell two artists apart.
    pub tags: Vec<String>,
}

/// Everything the resolver found about one artist, ready to store.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArtistMetadata {
    pub musicbrainz_artist_id: String,
    pub name: String,
    pub disambiguation: Option<String>,
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub begin_year: Option<i32>,
    pub end_year: Option<i32>,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub links: Vec<ArtistLink>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistAddRequest {
    pub musicbrainz_artist_id: String,
    pub note: Option<String>,
}

/// Changes to an artist on your list. An absent field is left alone.
///
/// `status` and `verdict` travel together: whenever `status` is sent, `verdict` is taken as
/// given, so marking someone checked without a verdict clears an old one, and moving them
/// back to the list clears it too. An empty `note` removes the note.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistUpdateRequest {
    pub note: Option<String>,
    pub status: Option<ArtistStatus>,
    pub verdict: Option<ArtistVerdict>,
}

/// Where an album from a discography already is in your library.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryMatch {
    pub release_id: String,
    pub status: ReleaseStatus,
    pub rating: Option<f64>,
}

/// One album of an artist's discography, and whether you already have it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscographyEntry {
    #[serde(flatten)]
    pub album: AlbumCandidate,
    pub library: Option<LibraryMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `frontend/src/types/index.ts` reads these keys. The album's fields sit beside
    /// `library`, not inside an `album` object.
    #[test]
    fn a_discography_entry_serializes_flat_in_camel_case() {
        let entry = DiscographyEntry {
            album: AlbumCandidate {
                musicbrainz_release_group_id: "g".into(),
                artist: None,
                title: Some("Nightmare".into()),
                release_year: Some(2010),
                primary_type: Some("Album".into()),
                secondary_types: vec![],
                disambiguation: None,
                album_art_url: "https://x".into(),
            },
            library: Some(LibraryMatch { release_id: "r".into(), status: ReleaseStatus::Listened, rating: Some(4.0) }),
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["musicbrainzReleaseGroupId"], "g");
        assert_eq!(json["releaseYear"], 2010);
        assert_eq!(json["library"]["releaseId"], "r");
        assert_eq!(json["library"]["status"], "LISTENED");
        assert!(json.get("album").is_none());
    }

    #[test]
    fn artist_status_and_verdict_use_the_frontend_words() {
        assert_eq!(serde_json::to_value(ArtistStatus::ToCheck).unwrap(), "TO_CHECK");
        assert_eq!(serde_json::to_value(ArtistVerdict::NotForMe).unwrap(), "NOT_FOR_ME");
        let req: ArtistUpdateRequest =
            serde_json::from_str(r#"{"status":"CHECKED","verdict":"LIKED","note":"x"}"#).unwrap();
        assert_eq!(req.status, Some(ArtistStatus::Checked));
        assert_eq!(req.verdict, Some(ArtistVerdict::Liked));
        let add: ArtistAddRequest = serde_json::from_str(r#"{"musicbrainzArtistId":"m"}"#).unwrap();
        assert_eq!(add.musicbrainz_artist_id, "m");
    }
}
