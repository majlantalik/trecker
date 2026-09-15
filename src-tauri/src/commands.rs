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

/// Up to ten albums matching what was typed into quick add.
#[tauri::command]
pub async fn releases_search(
    resolver: State<'_, Resolver>,
    query: String,
) -> AppResult<Vec<AlbumCandidate>> {
    resolver.search_albums(&query).await
}

/// Everything known about the album chosen from a search: genres, country and cover, which
/// the search results leave out.
#[tauri::command]
pub async fn releases_lookup(
    resolver: State<'_, Resolver>,
    release_group_id: String,
) -> AppResult<ResolvedMetadata> {
    let id = release_group_id.trim();
    if id.is_empty() {
        return Err(AppError::Invalid("no album to look up".into()));
    }
    resolver.lookup(id).await
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

// ---------------------------------------------------------------- artists

/// Up to ten artists matching what was typed.
#[tauri::command]
pub async fn artists_search(
    resolver: State<'_, Resolver>,
    query: String,
) -> AppResult<Vec<ArtistCandidate>> {
    resolver.search_artists(&query).await
}

/// Puts an artist chosen from a search on your list, looking them up first.
#[tauri::command]
pub async fn artists_add(
    db: State<'_, Db>,
    resolver: State<'_, Resolver>,
    request: ArtistAddRequest,
) -> AppResult<Artist> {
    // Already there: no need to spend two MusicBrainz requests to learn the same thing.
    if let Some(existing) = repo::artists::find_tracked(&db.pool, &request.musicbrainz_artist_id).await? {
        return Ok(existing);
    }
    let found = resolver.lookup_artist(&request.musicbrainz_artist_id).await?;
    repo::artists::add(&db.pool, found, request.note).await
}

#[tauri::command]
pub async fn artists_list(db: State<'_, Db>) -> AppResult<Vec<Artist>> {
    repo::artists::list(&db.pool).await
}

#[tauri::command]
pub async fn artists_get(db: State<'_, Db>, id: String) -> AppResult<Artist> {
    repo::artists::get(&db.pool, &id).await
}

#[tauri::command]
pub async fn artists_update(
    db: State<'_, Db>,
    id: String,
    request: ArtistUpdateRequest,
) -> AppResult<Artist> {
    repo::artists::update(&db.pool, &id, request).await
}

#[tauri::command]
pub async fn artists_delete(db: State<'_, Db>, id: String) -> AppResult<()> {
    repo::artists::delete(&db.pool, &id).await
}

/// An artist's albums and EPs from MusicBrainz, each marked with where it already is in your
/// library.
#[tauri::command]
pub async fn artists_discography(
    db: State<'_, Db>,
    resolver: State<'_, Resolver>,
    id: String,
) -> AppResult<Vec<DiscographyEntry>> {
    let artist = repo::artists::get(&db.pool, &id).await?;
    let mbid = artist
        .musicbrainz_artist_id
        .ok_or_else(|| AppError::Invalid("this artist has no MusicBrainz id".into()))?;
    let albums = resolver.artist_discography(&mbid).await?;
    attach_library(&db.pool, albums).await
}

/// Marks each album with your library's copy of it. Separate from the command so it can be
/// tested without the network.
pub async fn attach_library(
    pool: &sqlx::SqlitePool,
    albums: Vec<AlbumCandidate>,
) -> AppResult<Vec<DiscographyEntry>> {
    let ids: Vec<String> = albums.iter().map(|a| a.musicbrainz_release_group_id.clone()).collect();
    let mut matches = repo::artists::library_matches(pool, &ids).await?;
    Ok(albums
        .into_iter()
        .map(|album| DiscographyEntry {
            library: matches.remove(&album.musicbrainz_release_group_id),
            album,
        })
        .collect())
}

// ---------------------------------------------------------------- settings and desktop

#[tauri::command]
pub async fn settings_get(
    settings: State<'_, crate::settings::SettingsStore>,
) -> AppResult<crate::settings::Settings> {
    Ok(settings.get())
}

/// Applies new settings, then saves them. Applied first so that a tray icon this desktop
/// cannot show is reported as an error and not saved as a choice that silently does
/// nothing.
#[tauri::command]
pub async fn settings_update(
    app: tauri::AppHandle,
    settings: State<'_, crate::settings::SettingsStore>,
    request: crate::settings::Settings,
) -> AppResult<crate::settings::Settings> {
    crate::desktop::apply_close_action(&app, request.close_action)?;
    settings.replace(request)
}

/// What this launch was started to do, such as `quick-add`, handed over once.
#[tauri::command]
pub async fn app_take_launch_action(
    launch: State<'_, crate::desktop::LaunchAction>,
) -> AppResult<Option<String>> {
    Ok(launch.take())
}

/// The command a person binds to a desktop shortcut to open quick add.
#[tauri::command]
pub async fn app_quick_add_command() -> AppResult<String> {
    let exe = std::env::current_exe()
        .map_err(|e| AppError::Internal(format!("could not find the app's own path: {e}")))?;
    Ok(crate::desktop::quick_add_command(
        &exe,
        std::env::var_os("APPIMAGE").as_deref(),
        std::env::var_os("PATH").as_deref(),
    ))
}
