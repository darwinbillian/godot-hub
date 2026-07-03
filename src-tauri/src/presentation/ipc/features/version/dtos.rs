use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::version::{Version, VersionStatus, VersionUpdateEventArgs},
    presentation::ipc::dtos::ErrorDto,
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
    Failed { error: ErrorDto },
}

#[derive(Serialize, Debug)]
pub struct VersionUpdateEventArgsDto {
    name: String,
    flavor: String,
    status: VersionStatusDto,
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

impl<V> From<V> for VersionStatusDto
where
    V: Borrow<VersionStatus>,
{
    fn from(value: V) -> Self {
        let value = value.borrow();
        match value {
            VersionStatus::Available => Self::Available,
            VersionStatus::Installing => Self::Installing,
            VersionStatus::Installed => Self::Installed,
            VersionStatus::Failed(e) => Self::Failed {
                error: e.as_ref().into(),
            },
        }
    }
}

impl<V> From<V> for VersionUpdateEventArgsDto
where
    V: Borrow<VersionUpdateEventArgs>,
{
    fn from(value: V) -> Self {
        let value = value.borrow();
        Self {
            name: value.name.clone(),
            flavor: value.flavor.clone(),
            status: VersionStatusDto::from(&value.status),
        }
    }
}
