use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::release::{Release, ReleaseStatus, ReleaseUpdateEventArgs},
    presentation::ipc::dtos::ErrorDto,
};

#[derive(Serialize, Debug)]
pub struct ReleaseDto {
    name: String,
    flavor: String,
    release_notes: String,
    status: ReleaseStatusDto,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReleaseStatusDto {
    Available,
    Installing,
    Installed,
    Failed { error: ErrorDto },
}

#[derive(Serialize, Debug)]
pub struct ReleaseUpdateEventArgsDto {
    name: String,
    flavor: String,
    status: ReleaseStatusDto,
}

impl From<Release> for ReleaseDto {
    fn from(value: Release) -> Self {
        Self {
            name: value.name,
            flavor: value.flavor,
            release_notes: value.release_notes,
            status: value.status.into(),
        }
    }
}

impl<V> From<V> for ReleaseStatusDto
where
    V: Borrow<ReleaseStatus>,
{
    fn from(value: V) -> Self {
        let value = value.borrow();
        match value {
            ReleaseStatus::Available => Self::Available,
            ReleaseStatus::Installing => Self::Installing,
            ReleaseStatus::Installed => Self::Installed,
            ReleaseStatus::Failed(e) => Self::Failed {
                error: e.as_ref().into(),
            },
        }
    }
}

impl<V> From<V> for ReleaseUpdateEventArgsDto
where
    V: Borrow<ReleaseUpdateEventArgs>,
{
    fn from(value: V) -> Self {
        let value = value.borrow();
        Self {
            name: value.name.clone(),
            flavor: value.flavor.clone(),
            status: ReleaseStatusDto::from(&value.status),
        }
    }
}
