use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::release::{Release, ReleaseStatus},
    presentation::ipc::features::install::dtos::InstallDto,
};

#[derive(Serialize, Debug)]
pub struct ReleaseDto {
    id: String,
    name: String,
    version: String,
    flavor: String,
    variant: String,
    release_notes: String,
    status: ReleaseStatusDto,
    install: Option<InstallDto>,
}

impl From<Release> for ReleaseDto {
    fn from(value: Release) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version.to_string(),
            flavor: value.flavor.to_string(),
            variant: value.variant.to_string(),
            release_notes: value.release_notes,
            status: value.status.into(),
            install: value.install.map(Into::into),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReleaseStatusDto {
    Available,
    Unavailable,
}

impl<R> From<R> for ReleaseStatusDto
where
    R: Borrow<ReleaseStatus>,
{
    fn from(value: R) -> Self {
        let value = value.borrow();
        match value {
            ReleaseStatus::Available => Self::Available,
            ReleaseStatus::Unavailable => Self::Unavailable,
        }
    }
}
