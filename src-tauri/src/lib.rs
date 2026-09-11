mod commands;
mod db;
mod domain;
mod error;
mod repo;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            commands::releases_search_catalog,
            commands::genres_list,
            commands::stats_activity,
            commands::stats_by_genre,
            commands::stats_by_country,
            commands::stats_top_rated,
            commands::stats_year_end,
            commands::settings_db_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
