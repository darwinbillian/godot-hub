mod error;
mod event;
mod godot_website;
mod ipc;
mod services;
mod state;
mod utils;

use std::sync::Arc;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tauri::Manager;

use crate::{
    godot_website::{GodotWebsiteClient, GodotWebsiteVersionProvider},
    ipc::features::{
        install::emitter::{InstallRemoveEmitter, InstallUpdateEmitter},
        version::emitter::VersionUpdateEmitter,
    },
    services::{
        download::DownloadService, install::InstallService, task::TaskService,
        version::VersionService,
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

            let download_service =
                DownloadService::new(client.clone(), local_data_dir.join("downloads"));

            let task_service = TaskService::new();

            let install_service = InstallService::new(
                download_service,
                task_service.clone(),
                local_data_dir.join("installs"),
            );

            install_service
                .update_event()
                .subscribe(InstallUpdateEmitter::new(app.handle().clone()));

            install_service
                .remove_event()
                .subscribe(InstallRemoveEmitter::new(app.handle().clone()));

            let version_service = VersionService::new(version_provider, install_service.clone());

            version_service
                .update_event()
                .subscribe(VersionUpdateEmitter::new(app.handle().clone()));

            let state = AppState {
                install_service,
                version_service,
            };
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::show,
            ipc::features::install::commands::installs_install,
            ipc::features::install::commands::installs_list,
            ipc::features::install::commands::installs_launch,
            ipc::features::install::commands::installs_uninstall,
            ipc::features::install::commands::installs_reveal,
            ipc::features::version::commands::versions_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
