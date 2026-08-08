use tauri::State;

use super::dtos::InstallDto;
use crate::{
    domain::models::version::{Flavor, Variant, Version},
    presentation::ipc::{dtos::ErrorDto, features::install::dtos::InstallFilterDto},
    state::AppState,
};

#[tauri::command(rename = "installs::install")]
pub async fn installs_install(
    state: State<'_, AppState>,
    version: String,
    flavor: String,
    variant: String,
) -> Result<(), ErrorDto> {
    let version = version.parse::<Version>()?;
    let flavor = flavor.parse::<Flavor>()?;
    let variant = variant.parse::<Variant>()?;
    state
        .install_service
        .install(version, flavor, variant)
        .await?;
    Ok(())
}

#[tauri::command(rename = "installs::list")]
pub async fn installs_list(
    state: State<'_, AppState>,
    filter: Option<InstallFilterDto>,
) -> Result<Vec<InstallDto>, ErrorDto> {
    let filter = filter.map(|filter| filter.try_into()).transpose()?;

    let installs = state
        .install_service
        .list(filter)
        .await?
        .into_iter()
        .map(InstallDto::from)
        .collect::<Vec<InstallDto>>();

    Ok(installs)
}

#[tauri::command(rename = "installs::launch")]
pub async fn installs_launch(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.installation_service.get(&id).await?;
    install.launch()?;
    Ok(())
}

#[tauri::command(rename = "installs::uninstall")]
pub async fn installs_uninstall(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.installation_service.get(&id).await?;
    install.uninstall().await?;
    Ok(())
}

#[tauri::command(rename = "installs::reveal")]
pub async fn installs_reveal(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    let install = state.installation_service.get(&id).await?;
    install.reveal()?;
    Ok(())
}

#[tauri::command(rename = "installs::cancel")]
pub async fn installs_cancel(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    if let Some(task) = state.install_service.task_service().get(&id) {
        task.cancel();
    }
    Ok(())
}

#[tauri::command(rename = "installs::pause")]
pub async fn installs_pause(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    if let Some(task) = state.install_service.task_service().get(&id) {
        task.pause();
    }
    Ok(())
}

#[tauri::command(rename = "installs::resume")]
pub async fn installs_resume(state: State<'_, AppState>, id: String) -> Result<(), ErrorDto> {
    if let Some(task) = state.install_service.task_service().get(&id) {
        task.resume();
    }
    Ok(())
}
