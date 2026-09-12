//! The command surface.
//!
//! These commands map one to one onto the functions in `frontend/src/api/*.ts`.
//! `frontend/src/api/commands.test.ts` asserts every name and argument key.

use crate::db::{Db, DbInfo};
use crate::domain::*;
use crate::error::{AppError, AppResult};
use crate::library;
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

    // By id when the album has one, which is every album added from a search. A second
    // search could land on a different album entirely; a lookup cannot. Only a release
    // typed in by hand has no id, and a search is the only way to find it anything.
    let found = match repo::releases::release_group_id(pool, id).await? {
        Some(group) => resolver.lookup(&group).await?,
        None => {
            resolver
                .resolve(ResolveRequest {
                    query: Some(format!("{} - {}", current.artist, current.title)),
                    url: None,
                })
                .await?
        }
    };

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

// ---------------------------------------------------------------- export and import

/// What an export wrote, so the interface can say so without reading the file back.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    pub path: String,
    pub format: library::Format,
    pub release_count: usize,
    pub bytes: usize,
}

/// Writes the whole library to a path the user has already chosen in a save dialog.
///
/// Rust writes the file rather than handing the text back for the frontend to save. That
/// keeps the filesystem permission to exactly this one command and means the format never
/// has to survive a trip through the webview.
#[tauri::command]
pub async fn library_export(
    db: State<'_, Db>,
    app: tauri::AppHandle,
    path: String,
    format: library::Format,
) -> AppResult<ExportSummary> {
    let (path, format) = library::export_path(&path, format);
    let version = app.package_info().version.to_string();
    let releases = collect_export(&db.pool).await?;
    let text = library::render(&releases, format, &version, &repo::now())?;

    std::fs::write(&path, text.as_bytes())
        .map_err(|e| AppError::Internal(format!("could not write {path}: {e}")))?;

    Ok(ExportSummary {
        path,
        format,
        release_count: releases.len(),
        bytes: text.len(),
    })
}

/// Reads every tracked release into the file format's shape.
pub async fn collect_export(pool: &sqlx::SqlitePool) -> AppResult<Vec<library::ExportedRelease>> {
    Ok(repo::releases::export_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let r = row.release;
            library::ExportedRelease {
                artist: r.artist,
                title: r.title,
                release_year: r.release_year,
                country: r.country,
                album_art_url: r.album_art_url,
                musicbrainz_id: row.musicbrainz_release_group_id,
                genres: r.genres,
                streaming_links: r.streaming_links.into_iter().collect(),
                status: crate::repo::filter::status_str(r.status).to_string(),
                rating: r.rating,
                did_not_finish: r.did_not_finish,
                date_listened: r.date_listened,
                notes: r.notes,
                discovery_link: r.discovery_link,
                added_at: Some(r.created_at),
            }
        })
        .collect())
}

#[tauri::command]
pub async fn library_import(
    db: State<'_, Db>,
    path: String,
    mode: Option<library::ImportMode>,
) -> AppResult<library::ImportReport> {
    let text = std::fs::read_to_string(&path).map_err(|e| {
        AppError::Invalid(match e.kind() {
            std::io::ErrorKind::InvalidData => format!("{path} is not a UTF-8 text file"),
            _ => format!("could not read {path}: {e}"),
        })
    })?;

    import_library(&db.pool, &path, &text, mode.unwrap_or_default()).await
}

/// The body of `library_import`, free of Tauri state and the filesystem so it can be
/// tested against a temporary database.
pub async fn import_library(
    pool: &sqlx::SqlitePool,
    path: &str,
    text: &str,
    mode: library::ImportMode,
) -> AppResult<library::ImportReport> {
    let parsed = library::parse(text, library::detect(path, text))?;

    let mut report = library::ImportReport {
        rejected: parsed.rejected,
        ..Default::default()
    };

    for (row, release) in parsed.releases {
        // One release at a time, each in its own transaction. A row the database refuses
        // joins the rejected list instead of abandoning the rows after it: a partial
        // import that says what it did beats an all-or-nothing one that says why it did
        // nothing.
        match repo::releases::import_one(pool, &release, mode).await {
            Ok(repo::releases::ImportOutcome::Added) => report.added += 1,
            Ok(repo::releases::ImportOutcome::Overwritten) => report.overwritten += 1,
            Ok(repo::releases::ImportOutcome::Skipped) => report.skipped += 1,
            Err(e) => report.rejected.push(library::RejectedRow {
                row,
                artist: release.artist.clone(),
                title: release.title.clone(),
                reason: e.to_string(),
            }),
        }
    }

    report.rejected.sort_by_key(|r| r.row);
    Ok(report)
}

// ---------------------------------------------------------------- caches

#[tauri::command]
pub async fn cache_covers_info(covers: State<'_, crate::covers::CoverCache>) -> AppResult<crate::covers::CoverCacheInfo> {
    covers.info()
}

/// Returns what was removed. Covers download again as albums are shown.
#[tauri::command]
pub async fn cache_covers_clear(covers: State<'_, crate::covers::CoverCache>) -> AppResult<crate::covers::CoverCacheInfo> {
    covers.clear()
}

/// Clears the webview's own stored data: its HTTP cache, and in principle cookies and
/// browser storage. Safe because Trecker keeps nothing in browser storage. If that ever
/// changes, this button starts wiping user settings. See `CLAUDE.md`.
#[tauri::command]
pub async fn cache_webview_clear(window: tauri::WebviewWindow) -> AppResult<()> {
    window
        .clear_all_browsing_data()
        .map_err(|e| AppError::Internal(format!("could not clear webview data: {e}")))
}
