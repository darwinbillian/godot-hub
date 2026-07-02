use tauri::State;

use super::dtos::VersionDto;
use crate::{ipc::dtos::ErrorDto, state::AppState};

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<VersionDto>, ErrorDto> {
    let versions = state.version_service.list().await?;
    Ok(versions.into_iter().map(VersionDto::from).collect())
}
