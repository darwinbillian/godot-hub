use tauri::{AppHandle, Emitter, State};

use crate::{
    error::Error,
    services::{install::Install, version::Version},
    state::AppState,
};

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<Version>, Error> {
    let versions = state.version_service.list().await?;
    Ok(versions)
}

#[tauri::command]
pub async fn install(
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
pub async fn list_installs(state: State<'_, AppState>) -> Result<Vec<Install>, Error> {
    let installs = state.install_service.list().await?;
    Ok(installs)
}

#[tauri::command]
pub async fn launch(state: State<'_, AppState>, id: String) -> Result<(), Error> {
    state.install_service.launch(&id).await?;
    Ok(())
}

#[tauri::command]
pub async fn uninstall(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), Error> {
    state.install_service.uninstall(&id).await?;
    app.emit("update_installs", ())?;
    Ok(())
}
