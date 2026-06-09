mod error;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use crate::error::Error;

struct AppState {
    client: ClientWithMiddleware,
}

#[derive(Serialize, Deserialize, Debug)]
struct Version {
    name: String,
    flavor: String,
    release_date: String,
    release_notes: String,
    featured: Option<String>,
    releases: Option<Vec<VersionRelease>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VersionRelease {
    name: String,
    release_date: String,
    release_notes: String,
    release_version: Option<String>,
}

#[tauri::command]
async fn list_versions(state: State<'_, AppState>) -> Result<Vec<Version>, Error> {
    let request = state.client.get(
        "https://raw.githubusercontent.com/godotengine/godot-website/master/_data/versions.yml",
    );
    let response = request.send().await?.error_for_status()?;
    let bytes = response.bytes().await?;
    let versions = yaml_serde::from_slice::<Vec<Version>>(&bytes)?;
    Ok(versions)
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

            let state = AppState { client };
            app.manage(state);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
