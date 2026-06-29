use std::borrow::Borrow;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State, Window};

use crate::{
    error::Error,
    services::{
        install::Install,
        version::{Version, VersionStatus, VersionUpdateEventArgs},
    },
    state::AppState,
};

#[derive(Serialize, Debug)]
pub struct ErrorDto {
    message: String,
}

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
    Failed { error: ErrorDto },
}

#[derive(Serialize, Debug)]
pub struct VersionUpdateEventArgsDto {
    name: String,
    flavor: String,
    status: VersionStatusDto,
}

#[derive(Serialize, Debug)]
pub struct InstallDto {
    id: String,
    dir: String,
    version: String,
    flavor: String,
}

impl<E> From<E> for ErrorDto
where
    E: Borrow<Error>,
{
    fn from(value: E) -> Self {
        Self {
            message: value.borrow().to_string(),
        }
    }
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
            VersionStatus::Failed(e) => VersionStatusDto::Failed { error: e.into() },
        }
    }
}

impl From<VersionUpdateEventArgs> for VersionUpdateEventArgsDto {
    fn from(value: VersionUpdateEventArgs) -> Self {
        Self {
            name: value.version,
            flavor: value.flavor,
            status: value.status.into(),
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
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<VersionDto>, ErrorDto> {
    let versions = state.version_service.list().await?;
    Ok(versions.into_iter().map(VersionDto::from).collect())
}

#[tauri::command]
pub async fn install(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
    flavor: String,
) -> Result<(), ErrorDto> {
    state.install_service.install(&version, &flavor).await?;
    app.emit("update_installs", ()).map_err(Error::from)?;
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
pub async fn uninstall(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), ErrorDto> {
    let install = state.install_service.get(&id).await?;
    install.uninstall().await?;
    app.emit("update_installs", ()).map_err(Error::from)?;
    Ok(())
}

#[tauri::command]
pub async fn reveal(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.install_service.get(&id).await?;
    install.reveal().await?;
    Ok(())
}
