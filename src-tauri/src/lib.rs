mod error;
mod godot_website;
mod services;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tauri::{Manager, State};

use crate::{
    error::Error,
    godot_website::Version,
    services::install::{Install, InstallService},
};

struct AppState {
    client: ClientWithMiddleware,
    install_service: InstallService,
}

#[tauri::command]
async fn list_versions(state: State<'_, AppState>) -> Result<Vec<Version>, Error> {
    let versions = crate::godot_website::get_versions(&state.client).await?;
    Ok(versions)
}

#[tauri::command]
async fn install(state: State<'_, AppState>, version: String, flavor: String) -> Result<(), Error> {
    state.install_service.install(&version, &flavor).await?;
    Ok(())
}

#[tauri::command]
async fn list_installs(state: State<'_, AppState>) -> Result<Vec<Install>, Error> {
    let installs = state.install_service.list().await?;
    Ok(installs)
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

            let install_service = InstallService {
                client: client.clone(),
                downloads_dir: local_data_dir.join("downloads"),
                installs_dir: local_data_dir.join("installs"),
            };

            let state = AppState {
                client,
                install_service,
            };
            app.manage(state);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_versions,
            install,
            list_installs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
