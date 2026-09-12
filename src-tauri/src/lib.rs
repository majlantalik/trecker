mod commands;
mod covers;
mod db;
mod domain;
mod error;
mod library;
mod repo;
mod resolve;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Every album cover the webview shows comes through here. See `covers.rs`.
        .register_asynchronous_uri_scheme_protocol(covers::SCHEME, |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let path = request.uri().path().to_string();
            tauri::async_runtime::spawn(async move {
                let cache = app.state::<covers::CoverCache>();
                responder.respond(covers::respond(&cache, &path).await);
            });
        })
        .setup(|app| {
            // The database lives beside the app's other per-user state, which is
            // %APPDATA% on Windows, ~/Library/Application Support on macOS and
            // ~/.local/share on Linux. Tauri resolves the right one.
            let dir = app.path().app_data_dir()?;

            // Opening the database is the one startup step with no useful fallback: if
            // the schema cannot be created the app has nowhere to put anything, so fail
            // here rather than surfacing the same error on every command.
            let db = tauri::async_runtime::block_on(db::connect(&dir))?;
            app.manage(db);
            let version = app.package_info().version.to_string();
            app.manage(resolve::Resolver::new(&version));

            // The OS cache directory, not the data directory beside the database: covers are
            // derived from stored URLs, and a system cleaner removing them loses nothing.
            let covers = app.path().app_cache_dir()?.join("covers");
            app.manage(covers::CoverCache::new(covers, &version));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::releases_list,
            commands::releases_get,
            commands::releases_random,
            commands::releases_create,
            commands::releases_update,
            commands::releases_delete,
            commands::releases_resolve,
            commands::releases_search,
            commands::releases_lookup,
            commands::releases_refresh_metadata,
            commands::releases_search_catalog,
            commands::genres_list,
            commands::countries_list,
            commands::stats_activity,
            commands::stats_by_genre,
            commands::stats_by_country,
            commands::stats_top_rated,
            commands::stats_year_end,
            commands::info_db,
            commands::library_export,
            commands::library_import,
            commands::cache_covers_info,
            commands::cache_covers_clear,
            commands::cache_webview_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
