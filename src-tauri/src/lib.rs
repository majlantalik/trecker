use serde::Serialize;
use std::collections::HashMap;

/// Phase 0 spike only.
///
/// Mirrors the `Release` interface in `frontend/src/types/index.ts` field for field.
/// Its single job is to prove the Tauri command boundary serializes into the shape the
/// existing Vue components already consume, so that Phase 1 is a body swap and nothing
/// more. Delete this module once `commands/releases.rs` lands in Phase 3.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub artist: String,
    pub title: String,
    pub release_year: Option<i32>,
    pub album_art_url: Option<String>,
    pub status: String,
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

/// A local, offline placeholder so the spike exercises the `<img>` path without
/// depending on the network or on CSP rules for remote hosts.
fn placeholder_art() -> String {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="#00e5b0"/><stop offset="100%" stop-color="#13162a"/></linearGradient></defs><rect width="300" height="300" fill="url(#g)"/></svg>"##;
    format!(
        "data:image/svg+xml;utf8,{}",
        svg.replace('#', "%23").replace('"', "%22")
    )
}

#[tauri::command]
fn spike_releases() -> Vec<Release> {
    vec![
        Release {
            id: "11111111-1111-4111-8111-111111111111".into(),
            artist: "Godspeed You! Black Emperor".into(),
            title: "Lift Your Skinny Fists Like Antennas to Heaven".into(),
            release_year: Some(2000),
            album_art_url: Some(placeholder_art()),
            status: "LISTENED".into(),
            discovery_link: None,
            streaming_links: HashMap::new(),
            country: Some("CA".into()),
            rating: Some(4.5),
            did_not_finish: false,
            date_listened: Some("2026-09-01T19:30:00Z".into()),
            notes: Some("Long-form crescendo. Needs a full sitting.".into()),
            created_at: "2026-08-20T10:00:00Z".into(),
            genres: vec!["post-rock".into(), "experimental".into(), "drone".into()],
        },
        Release {
            // No art, no rating: exercises the placeholder and the empty-state branches.
            id: "22222222-2222-4222-8222-222222222222".into(),
            artist: "Ryo Fukui".into(),
            title: "Scenery".into(),
            release_year: Some(1976),
            album_art_url: None,
            status: "QUEUED".into(),
            discovery_link: None,
            streaming_links: HashMap::new(),
            country: Some("JP".into()),
            rating: None,
            did_not_finish: false,
            date_listened: None,
            notes: None,
            created_at: "2026-09-05T08:15:00Z".into(),
            genres: vec!["jazz".into()],
        },
        Release {
            // Long strings and a half-star rating: exercises truncation and the rating widget.
            id: "33333333-3333-4333-8333-333333333333".into(),
            artist: "Mount Kimbie".into(),
            title: "The Sunset Violent".into(),
            release_year: Some(2024),
            album_art_url: Some(placeholder_art()),
            status: "LISTENED".into(),
            discovery_link: None,
            streaming_links: HashMap::new(),
            country: Some("GB".into()),
            rating: Some(3.5),
            did_not_finish: true,
            date_listened: Some("2026-09-09T21:00:00Z".into()),
            notes: None,
            created_at: "2026-09-02T12:00:00Z".into(),
            genres: vec!["electronic".into(), "post-punk".into()],
        },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![spike_releases])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
