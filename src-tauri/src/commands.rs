//! The command surface.
//!
//! These fourteen commands map one to one onto the functions that used to live in
//! `frontend/src/api/*.ts`. The mapping is the contract for the whole migration: Phase 3
//! replaces the bodies with SQLite and Phase 4 replaces `releases_resolve` with real
//! providers, but the names, arguments and return shapes stay exactly as they are here.

use crate::db::{Db, DbInfo};
use crate::domain::*;
use crate::error::{AppError, AppResult};
use crate::store::Store;
use std::collections::HashMap;
use tauri::State;

// ---------------------------------------------------------------- settings

/// Diagnostics, not part of the release API. Surfaces where the database actually is and
/// what state it is in, which is the only way to tell from inside the app that the file
/// was created, migrated and opened with the pragmas we asked for.
#[tauri::command]
pub async fn settings_db_info(db: State<'_, Db>) -> AppResult<DbInfo> {
    db.info().await
}

// ---------------------------------------------------------------- releases

#[tauri::command]
pub fn releases_list(
    store: State<'_, Store>,
    params: Option<ReleaseFilterParams>,
) -> AppResult<PageResponse<Release>> {
    store.list(&params.unwrap_or_default())
}

#[tauri::command]
pub fn releases_get(store: State<'_, Store>, id: String) -> AppResult<Release> {
    store.get(&id)
}

#[tauri::command]
pub fn releases_random(store: State<'_, Store>) -> AppResult<Release> {
    store.random_queued()
}

#[tauri::command]
pub fn releases_create(store: State<'_, Store>, request: ReleaseRequest) -> AppResult<Release> {
    store.create(request)
}

#[tauri::command]
pub fn releases_update(
    store: State<'_, Store>,
    id: String,
    request: ReleaseUpdateRequest,
) -> AppResult<Release> {
    store.update(&id, request)
}

#[tauri::command]
pub fn releases_delete(store: State<'_, Store>, id: String) -> AppResult<()> {
    store.delete(&id)
}

/// Phase 4 replaces this with `resolve::MetadataResolver`. Until then it echoes the
/// input back so the Quick Add flow is exercisable end to end.
#[tauri::command]
pub fn releases_resolve(request: ResolveRequest) -> AppResult<ResolvedMetadata> {
    let raw = request
        .url
        .or(request.query)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Invalid("nothing to resolve".into()))?;

    // "Artist - Title" is the shape the plain-text branch of the real resolver parses,
    // so the stub understands at least that much.
    let (artist, title) = match raw.split_once(" - ") {
        Some((a, t)) => (a.trim().to_string(), t.trim().to_string()),
        None => (String::new(), raw.trim().to_string()),
    };

    Ok(ResolvedMetadata {
        artist: (!artist.is_empty()).then_some(artist),
        title: (!title.is_empty()).then_some(title),
        streaming_links: HashMap::new(),
        ..Default::default()
    })
}

#[tauri::command]
pub fn releases_search_catalog(
    store: State<'_, Store>,
    q: String,
    limit: Option<u32>,
) -> AppResult<Vec<ResolvedMetadata>> {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let params = ReleaseFilterParams {
        search: Some(q),
        size: Some(limit.unwrap_or(10)),
        ..Default::default()
    };
    Ok(store
        .list(&params)?
        .content
        .into_iter()
        .map(|r| ResolvedMetadata {
            artist: Some(r.artist),
            title: Some(r.title),
            release_year: r.release_year,
            album_art_url: r.album_art_url,
            country: r.country,
            streaming_links: r.streaming_links,
            genres: r.genres,
            spotify_id: None,
            musicbrainz_id: None,
        })
        .collect())
}

// ---------------------------------------------------------------- genres

#[tauri::command]
pub fn genres_list(store: State<'_, Store>) -> AppResult<Vec<String>> {
    store.genres()
}

// ---------------------------------------------------------------- stats

#[tauri::command]
pub fn stats_activity(store: State<'_, Store>) -> AppResult<Vec<ActivityDataPoint>> {
    let mut counts: HashMap<(i32, u32), u64> = HashMap::new();
    for r in store.listened()? {
        if let Some((y, m)) = r.date_listened.as_deref().and_then(year_month) {
            *counts.entry((y, m)).or_default() += 1;
        }
    }
    let mut out: Vec<ActivityDataPoint> = counts
        .into_iter()
        .map(|((year, month), count)| ActivityDataPoint { year, month, count })
        .collect();
    out.sort_by_key(|d| (d.year, d.month));
    Ok(out)
}

#[tauri::command]
pub fn stats_by_genre(store: State<'_, Store>) -> AppResult<Vec<BreakdownItem>> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for r in store.listened()? {
        for g in r.genres {
            *counts.entry(g).or_default() += 1;
        }
    }
    Ok(to_breakdown(counts))
}

#[tauri::command]
pub fn stats_by_country(store: State<'_, Store>) -> AppResult<Vec<BreakdownItem>> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for r in store.listened()? {
        if let Some(c) = r.country {
            *counts.entry(c).or_default() += 1;
        }
    }
    Ok(to_breakdown(counts))
}

#[tauri::command]
pub fn stats_top_rated(store: State<'_, Store>, limit: Option<u32>) -> AppResult<Vec<Release>> {
    let mut listened = store.listened()?;
    listened.retain(|r| r.rating.is_some());
    listened.sort_by(|a, b| {
        b.rating
            .partial_cmp(&a.rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    listened.truncate(limit.unwrap_or(25) as usize);
    Ok(listened)
}

#[tauri::command]
pub fn stats_year_end(store: State<'_, Store>, year: Option<i32>) -> AppResult<Vec<YearEndEntry>> {
    let year = year.unwrap_or(2026);
    let mut listened: Vec<Release> = store
        .listened()?
        .into_iter()
        .filter(|r| {
            r.date_listened
                .as_deref()
                .and_then(year_month)
                .is_some_and(|(y, _)| y == year)
        })
        .filter(|r| r.rating.is_some())
        .collect();
    listened.sort_by(|a, b| {
        b.rating
            .partial_cmp(&a.rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(listened
        .into_iter()
        .enumerate()
        .map(|(i, release)| YearEndEntry {
            rank: i as u32 + 1,
            release,
        })
        .collect())
}

fn to_breakdown(counts: HashMap<String, u64>) -> Vec<BreakdownItem> {
    let mut out: Vec<BreakdownItem> = counts
        .into_iter()
        .map(|(label, count)| BreakdownItem { label, count })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then(a.label.cmp(&b.label)));
    out
}

/// Pulls year and month out of an ISO-8601 timestamp without a date library.
fn year_month(iso: &str) -> Option<(i32, u32)> {
    let y = iso.get(0..4)?.parse().ok()?;
    let m = iso.get(5..7)?.parse().ok()?;
    Some((y, m))
}
