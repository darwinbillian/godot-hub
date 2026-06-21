mod error;
mod godot_website;
mod services;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    error::Error,
    services::{
        download::DownloadService,
        install::{Install, InstallService},
        version::{Version, VersionService},
    },
};

struct AppState {
    install_service: InstallService,
    version_service: VersionService,
}

#[tauri::command]
async fn list_versions(state: State<'_, AppState>) -> Result<Vec<Version>, Error> {
    let versions = state.version_service.list().await?;
    Ok(versions)
}

#[tauri::command]
async fn install(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
    flavor: String,
) -> Result<(), Error> {
    state.install_service.install(&version, &flavor).await?;
    app.emit("update_installs", ())?;
    Ok(())
}

#[tauri::command]
async fn list_installs(state: State<'_, AppState>) -> Result<Vec<Install>, Error> {
    let installs = state.install_service.list().await?;
    Ok(installs)
}

#[tauri::command]
async fn launch(state: State<'_, AppState>, id: String) -> Result<(), Error> {
    state.install_service.launch(&id).await?;
    Ok(())
}

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
            list_versions,
            install,
            list_installs,
            launch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
