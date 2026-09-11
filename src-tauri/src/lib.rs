mod commands;
mod domain;
mod error;
mod store;

use store::Store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Store::new())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
