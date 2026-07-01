mod error;
mod event;
mod godot_website;
mod ipc;
mod services;
mod state;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tauri::{Emitter, Manager};

use crate::{
    ipc::dtos::{InstallRemoveEventArgsDto, InstallUpdateEventArgsDto, VersionUpdateEventArgsDto},
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

            let download_service =
                DownloadService::new(client.clone(), local_data_dir.join("downloads"));

            let task_service = TaskService::new();

            let install_service = InstallService::new(
                download_service,
                task_service.clone(),
                local_data_dir.join("installs"),
            );

            {
                let handle = app.handle().clone();
                install_service.update_event().subscribe(move |args| {
                    let _ = handle.emit("update_install", &InstallUpdateEventArgsDto::from(args));
                });
            }

            {
                let handle = app.handle().clone();
                install_service.remove_event().subscribe(move |args| {
                    let _ = handle.emit("remove_install", &InstallRemoveEventArgsDto::from(args));
                });
            }

            let version_service = VersionService::new(client.clone(), install_service.clone());

            {
                let handle = app.handle().clone();
                version_service.update_event().subscribe(move |args| {
                    let _ = handle.emit("update_version", &VersionUpdateEventArgsDto::from(args));
                });
            }

            let state = AppState {
                install_service,
                version_service,
            };
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::show,
            ipc::commands::list_versions,
            ipc::commands::install,
            ipc::commands::list_installs,
            ipc::commands::launch,
            ipc::commands::uninstall,
            ipc::commands::reveal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
