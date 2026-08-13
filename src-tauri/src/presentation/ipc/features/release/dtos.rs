use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::{
    application::services::release::{Release, ReleaseFilter, ReleaseStatus},
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

#[derive(Deserialize, Debug)]
pub struct ReleaseFilterDto {
    flavor: Option<String>,
    variant: Option<String>,
}

impl TryFrom<ReleaseFilterDto> for ReleaseFilter {
    type Error = anyhow::Error;

    fn try_from(value: ReleaseFilterDto) -> Result<Self, Self::Error> {
        let flavor = value.flavor.map(|flavor| flavor.parse()).transpose()?;
        let variant = value.variant.map(|variant| variant.parse()).transpose()?;
        let result = Self { flavor, variant };
        Ok(result)
    }
}
