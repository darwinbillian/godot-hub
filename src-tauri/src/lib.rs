mod application;
mod infrastructure;
mod presentation;
mod state;

use std::sync::Arc;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tauri::Manager;

use crate::{
    application::services::{
        download::DownloadService, install::InstallService, task::TaskService,
        version::VersionService,
    },
    infrastructure::godot_website::{
        client::GodotWebsiteClient,
        providers::{GodotWebsiteDownloadProvider, GodotWebsiteVersionProvider},
    },
    presentation::ipc::features::{
        install::events::{InstallRemoveEventEmitter, InstallUpdateEventEmitter},
        version::events::VersionUpdateEventEmitter,
    },
    state::AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let window = app.get_webview_window("main").expect("no main window");
            let _ = window.unminimize();
            let _ = window.set_focus();
        }))
        .setup(|app| {
            let local_data_dir = app.path().local_data_dir()?.join("Godot Hub");

            let client = ClientBuilder::new(Client::new())
                .with(Cache(HttpCache {
                    mode: CacheMode::Default,
                    manager: CACacheManager::new(local_data_dir.join("cache"), false),
                    options: HttpCacheOptions::default(),
                }))
                .build();

            let godot_website = GodotWebsiteClient::new(client.clone());

            let version_provider = Arc::new(GodotWebsiteVersionProvider::new(godot_website));

            let download_provider = Arc::new(GodotWebsiteDownloadProvider::new());

            let download_service =
                DownloadService::new(client.clone(), local_data_dir.join("downloads"));

            let task_service = TaskService::new();

            let install_service = InstallService::new(
                download_provider,
                download_service,
                task_service.clone(),
                local_data_dir.join("installs"),
            );

            install_service
                .update_event()
                .subscribe(InstallUpdateEventEmitter::new(app.handle().clone()));

            install_service
                .remove_event()
                .subscribe(InstallRemoveEventEmitter::new(app.handle().clone()));

            let version_service = VersionService::new(version_provider, install_service.clone());

            version_service
                .update_event()
                .subscribe(VersionUpdateEventEmitter::new(app.handle().clone()));

            let state = AppState {
                install_service,
                version_service,
            };
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            presentation::ipc::commands::show,
            presentation::ipc::features::install::commands::installs_install,
            presentation::ipc::features::install::commands::installs_list,
            presentation::ipc::features::install::commands::installs_launch,
            presentation::ipc::features::install::commands::installs_uninstall,
            presentation::ipc::features::install::commands::installs_reveal,
            presentation::ipc::features::version::commands::versions_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
