use tauri::State;

use super::dtos::VersionDto;
use crate::{presentation::ipc::dtos::ErrorDto, state::AppState};

#[tauri::command(rename = "versions::list")]
pub async fn versions_list(state: State<'_, AppState>) -> Result<Vec<VersionDto>, ErrorDto> {
    let versions = state.version_service.list().await?;
    Ok(versions.into_iter().map(VersionDto::from).collect())
}
