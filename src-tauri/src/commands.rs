//! The command surface.
//!
//! These fourteen commands map one to one onto the functions that used to live in
//! `frontend/src/api/*.ts`, plus one diagnostics command for the Settings view. Phase 4
//! replaces `releases_resolve` with real providers; nothing else here should move.

use crate::db::{Db, DbInfo};
use crate::domain::*;
use crate::error::{AppError, AppResult};
use crate::repo;
use std::collections::HashMap;
use tauri::State;

// ---------------------------------------------------------------- releases

#[tauri::command]
pub async fn releases_list(
    db: State<'_, Db>,
    params: Option<ReleaseFilterParams>,
) -> AppResult<PageResponse<Release>> {
    repo::releases::list(&db.pool, &params.unwrap_or_default()).await
}

#[tauri::command]
pub async fn releases_get(db: State<'_, Db>, id: String) -> AppResult<Release> {
    repo::releases::get(&db.pool, &id).await
}

#[tauri::command]
pub async fn releases_random(db: State<'_, Db>) -> AppResult<Release> {
    repo::releases::random_queued(&db.pool).await
}

#[tauri::command]
pub async fn releases_create(db: State<'_, Db>, request: ReleaseRequest) -> AppResult<Release> {
    repo::releases::create(&db.pool, request).await
}

#[tauri::command]
pub async fn releases_update(
    db: State<'_, Db>,
    id: String,
    request: ReleaseUpdateRequest,
) -> AppResult<Release> {
    repo::releases::update(&db.pool, &id, request).await
}

#[tauri::command]
pub async fn releases_delete(db: State<'_, Db>, id: String) -> AppResult<()> {
    repo::releases::delete(&db.pool, &id).await
}

/// Phase 4 replaces this with the real providers. Until then it parses the one input
/// shape the plain-text branch of the resolver understands, so Quick Add works.
#[tauri::command]
pub async fn releases_resolve(request: ResolveRequest) -> AppResult<ResolvedMetadata> {
    let raw = request
        .url
        .or(request.query)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Invalid("nothing to resolve".into()))?;

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
pub async fn releases_search_catalog(
    db: State<'_, Db>,
    q: String,
    limit: Option<u32>,
) -> AppResult<Vec<ResolvedMetadata>> {
    repo::releases::search_catalog(&db.pool, &q, limit.unwrap_or(10)).await
}

// ---------------------------------------------------------------- genres

#[tauri::command]
pub async fn genres_list(db: State<'_, Db>) -> AppResult<Vec<String>> {
    repo::releases::list_genres(&db.pool).await
}

// ---------------------------------------------------------------- stats

#[tauri::command]
pub async fn stats_activity(db: State<'_, Db>) -> AppResult<Vec<ActivityDataPoint>> {
    repo::stats::activity(&db.pool).await
}

#[tauri::command]
pub async fn stats_by_genre(db: State<'_, Db>) -> AppResult<Vec<BreakdownItem>> {
    repo::stats::by_genre(&db.pool).await
}

#[tauri::command]
pub async fn stats_by_country(db: State<'_, Db>) -> AppResult<Vec<BreakdownItem>> {
    repo::stats::by_country(&db.pool).await
}

#[tauri::command]
pub async fn stats_top_rated(db: State<'_, Db>, limit: Option<u32>) -> AppResult<Vec<Release>> {
    repo::stats::top_rated(&db.pool, limit.unwrap_or(25)).await
}

#[tauri::command]
pub async fn stats_year_end(db: State<'_, Db>, year: Option<i32>) -> AppResult<Vec<YearEndEntry>> {
    // The frontend sends no year on first load; "this year" is the sensible default and
    // deriving it here keeps the frontend from having to know.
    let year = year.unwrap_or_else(current_year);
    repo::stats::year_end(&db.pool, year).await
}

// ---------------------------------------------------------------- settings

/// Diagnostics, not part of the release API. Surfaces where the database actually is and
/// what state it is in, which is the only way to tell from inside the app that the file
/// was created, migrated and opened with the pragmas we asked for.
#[tauri::command]
pub async fn settings_db_info(db: State<'_, Db>) -> AppResult<DbInfo> {
    db.info().await
}

fn current_year() -> i32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    repo::now_year(secs)
}
