use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::release::{Release, ReleaseStatus},
    presentation::ipc::features::install::dtos::InstallDto,
};

#[derive(Serialize, Debug)]
pub struct ReleaseDto {
    name: String,
    flavor: String,
    release_notes: String,
    status: ReleaseStatusDto,
    install: Option<InstallDto>,
}

impl From<Release> for ReleaseDto {
    fn from(value: Release) -> Self {
        Self {
            name: value.name,
            flavor: value.flavor,
            release_notes: value.release_notes,
            status: value.status.into(),
            install: value.install.map(InstallDto::from),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReleaseStatusDto {
    Available,
}

impl<R> From<R> for ReleaseStatusDto
where
    R: Borrow<ReleaseStatus>,
{
    fn from(value: R) -> Self {
        let value = value.borrow();
        match value {
            ReleaseStatus::Available => Self::Available,
        }
    }
}
