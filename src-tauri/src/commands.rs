use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State, Window};

use crate::{
    error::Error,
    services::{
        install::Install,
        version::{Version, VersionStatus},
    },
    state::AppState,
};

#[derive(Serialize, Debug)]
pub struct VersionDto {
    name: String,
    flavor: String,
    release_notes: String,
    status: VersionStatusDto,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VersionStatusDto {
    Available,
    Installing,
    Installed,
    Failed { error: Arc<Error> },
}

#[derive(Serialize, Debug)]
pub struct InstallDto {
    id: String,
    dir: String,
    version: String,
    flavor: String,
}

impl From<Version> for VersionDto {
    fn from(value: Version) -> Self {
        Self {
            name: value.name,
            flavor: value.flavor,
            release_notes: value.release_notes,
            status: value.status.into(),
        }
    }
}

impl From<VersionStatus> for VersionStatusDto {
    fn from(value: VersionStatus) -> Self {
        match value {
            VersionStatus::Available => VersionStatusDto::Available,
            VersionStatus::Installing => VersionStatusDto::Installing,
            VersionStatus::Installed => VersionStatusDto::Installed,
            VersionStatus::Failed(e) => VersionStatusDto::Failed { error: e },
        }
    }
}

impl From<Install> for InstallDto {
    fn from(value: Install) -> Self {
        Self {
            id: value.id,
            dir: value.dir.to_string_lossy().into_owned(),
            version: value.metadata.version,
            flavor: value.metadata.flavor,
        }
    }
}

#[tauri::command]
pub async fn show(window: Window) {
    window.show().unwrap()
}

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<VersionDto>, Error> {
    let versions = state.version_service.list().await?;
    Ok(versions.into_iter().map(VersionDto::from).collect())
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
pub async fn list_installs(state: State<'_, AppState>) -> Result<Vec<InstallDto>, Error> {
    let installs = state.install_service.list().await?;
    Ok(installs.into_iter().map(InstallDto::from).collect())
}

#[tauri::command]
pub async fn launch(state: State<'_, AppState>, id: String) -> Result<(), Error> {
    let install = state.install_service.get(&id).await?;
    install.launch().await?;
    Ok(())
}

#[tauri::command]
pub async fn uninstall(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), Error> {
    let install = state.install_service.get(&id).await?;
    install.uninstall().await?;
    app.emit("update_installs", ())?;
    Ok(())
}

#[tauri::command]
pub async fn reveal(state: State<'_, AppState>, id: String) -> Result<(), Error> {
    let install = state.install_service.get(&id).await?;
    install.reveal().await?;
    Ok(())
}
