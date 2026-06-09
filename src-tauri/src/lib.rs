mod error;
mod godot_website;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tauri::{Manager, State};

use crate::{error::Error, godot_website::Version};

struct AppState {
    client: ClientWithMiddleware,
}

#[tauri::command]
async fn list_versions(state: State<'_, AppState>) -> Result<Vec<Version>, Error> {
    let versions = crate::godot_website::get_versions(&state.client).await?;
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
