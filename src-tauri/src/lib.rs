use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use thiserror::Error;

struct AppState {
    client: ClientWithMiddleware,
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Yaml(#[from] yaml_serde::Error),
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

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
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
            app.manage(AppState {
                client: ClientBuilder::new(Client::new())
                    .with(Cache(HttpCache {
                        mode: CacheMode::Default,
                        manager: CACacheManager::new(
                            app.path().app_local_data_dir()?.join("cache"),
                            false,
                        ),
                        options: HttpCacheOptions::default(),
                    }))
                    .build(),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
