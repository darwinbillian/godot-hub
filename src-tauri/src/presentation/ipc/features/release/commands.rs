use tauri::State;

use super::dtos::ReleaseDto;
use crate::{presentation::ipc::dtos::ErrorDto, state::AppState};

#[tauri::command(rename = "releases::list")]
pub async fn releases_list(state: State<'_, AppState>) -> Result<Vec<ReleaseDto>, ErrorDto> {
    let releases = state.release_service.list().await?;
    Ok(releases.into_iter().map(ReleaseDto::from).collect())
}
