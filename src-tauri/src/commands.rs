//! The command surface.
//!
//! These commands map one to one onto the functions in `frontend/src/api/*.ts`.
//! `frontend/src/api/commands.test.ts` asserts every name and argument key.

use crate::db::{Db, DbInfo};
use crate::domain::*;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::resolve::Resolver;
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

#[tauri::command]
pub async fn releases_resolve(
    resolver: State<'_, Resolver>,
    request: ResolveRequest,
) -> AppResult<ResolvedMetadata> {
    resolver.resolve(request).await
}

#[tauri::command]
pub async fn releases_refresh_metadata(
    db: State<'_, Db>,
    resolver: State<'_, Resolver>,
    id: String,
) -> AppResult<Release> {
    refresh_metadata(&db.pool, &resolver, &id).await
}

/// The body of `releases_refresh_metadata`, free of Tauri state so it can be tested.
pub async fn refresh_metadata(
    pool: &sqlx::SqlitePool,
    resolver: &Resolver,
    id: &str,
) -> AppResult<Release> {
    let current = repo::releases::get(pool, id).await?;

    let found = resolver
        .resolve(ResolveRequest {
            query: Some(format!("{} - {}", current.artist, current.title)),
            url: None,
        })
        .await?;

    // A lookup that comes back with nothing recognisable must not wipe what is already
    // there. Refusing is better than overwriting a good record with an empty one.
    if found.artist.is_none() && found.title.is_none() {
        return Err(AppError::Resolve(format!(
            "no metadata found for '{} - {}'",
            current.artist, current.title
        )));
    }

    repo::releases::update(
        pool,
        id,
        ReleaseUpdateRequest {
            artist: found.artist,
            title: found.title,
            release_year: found.release_year,
            album_art_url: found.album_art_url,
            country: found.country,
            // Empty genres mean the providers had none, not that yours should go.
            genres: (!found.genres.is_empty()).then_some(found.genres),
            ..Default::default()
        },
    )
    .await
}

#[tauri::command]
pub async fn releases_search_catalog(
    db: State<'_, Db>,
    q: String,
    limit: Option<u32>,
) -> AppResult<Vec<ResolvedMetadata>> {
    repo::releases::search_catalog(&db.pool, &q, limit.unwrap_or(10)).await
}

// ---------------------------------------------------------------- genres and countries

#[tauri::command]
pub async fn genres_list(db: State<'_, Db>) -> AppResult<Vec<String>> {
    repo::releases::list_genres(&db.pool).await
}

#[tauri::command]
pub async fn countries_list(db: State<'_, Db>) -> AppResult<Vec<String>> {
    repo::releases::list_countries(&db.pool).await
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

// ---------------------------------------------------------------- info

/// Diagnostics, behind the Info view. Surfaces where the database actually is and
/// what state it is in, which is the only way to tell from inside the app that the file
/// was created, migrated and opened with the pragmas we asked for.
#[tauri::command]
pub async fn info_db(db: State<'_, Db>) -> AppResult<DbInfo> {
    db.info().await
}

fn current_year() -> i32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    repo::now_year(secs)
}
