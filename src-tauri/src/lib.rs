mod commands;
mod error;
mod godot_website;
mod services;
mod state;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tauri::Manager;

use crate::{
    services::{download::DownloadService, install::InstallService, version::VersionService},
    state::AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let local_data_dir = app.path().local_data_dir()?.join("Godot Hub");

            let client = ClientBuilder::new(Client::new())
                .with(Cache(HttpCache {
                    mode: CacheMode::Default,
                    manager: CACacheManager::new(local_data_dir.join("cache"), false),
                    options: HttpCacheOptions::default(),
                }))
                .build();

            let download_service = DownloadService {
                client: client.clone(),
                dir: local_data_dir.join("downloads"),
            };

            let install_service = InstallService {
                download_service,
                dir: local_data_dir.join("installs"),
            };

            let version_service = VersionService {
                client: client.clone(),
            };

            let state = AppState {
                install_service,
                version_service,
            };
            app.manage(state);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_versions,
            commands::install,
            commands::list_installs,
            commands::launch,
            commands::uninstall,
            commands::reveal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
