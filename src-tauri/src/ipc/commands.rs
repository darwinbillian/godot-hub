use tauri::{State, Window};

use super::dtos::{ErrorDto, InstallDto, VersionDto};
use crate::state::AppState;

#[tauri::command]
pub async fn show(window: Window) {
    window.show().unwrap()
}

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<VersionDto>, ErrorDto> {
    let versions = state.version_service.list().await?;
    Ok(versions.into_iter().map(VersionDto::from).collect())
}

#[tauri::command]
pub async fn install(
    state: State<'_, AppState>,
    version: String,
    flavor: String,
) -> Result<(), ErrorDto> {
    state.install_service.install(&version, &flavor).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_installs(state: State<'_, AppState>) -> Result<Vec<InstallDto>, ErrorDto> {
    let installs = state.install_service.list().await?;
    Ok(installs.into_iter().map(InstallDto::from).collect())
}

#[tauri::command]
pub async fn launch(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.install_service.get(&id).await?;
    install.launch().await?;
    Ok(())
}

#[tauri::command]
pub async fn uninstall(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.install_service.get(&id).await?;
    install.uninstall().await?;
    Ok(())
}

#[tauri::command]
pub async fn reveal(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.install_service.get(&id).await?;
    install.reveal().await?;
    Ok(())
}
