use tauri::State;

use super::dtos::ReleaseDto;
use crate::{
    presentation::ipc::{dtos::ErrorDto, features::release::dtos::ReleaseFilterDto},
    state::AppState,
};

#[tauri::command(rename = "releases::list")]
pub async fn releases_list(
    state: State<'_, AppState>,
    filter: Option<ReleaseFilterDto>,
) -> Result<Vec<ReleaseDto>, ErrorDto> {
    let filter = filter.map(|filter| filter.try_into()).transpose()?;

    let releases = state
        .release_service
        .list(filter)
        .await?
        .into_iter()
        .map(ReleaseDto::from)
        .collect::<Vec<ReleaseDto>>();

    Ok(releases)
}
