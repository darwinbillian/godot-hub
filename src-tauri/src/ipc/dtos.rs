use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    error::Error,
    services::{
        install::{
            Install, InstallRemoveEventArgs, InstallStatus, InstallUpdateEventArgs, Installation,
        },
        version::{Version, VersionStatus, VersionUpdateEventArgs},
    },
};

#[derive(Serialize, Debug)]
pub struct ErrorDto {
    message: String,
}

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

#[derive(Serialize, Debug)]
pub struct InstallDto {
    id: String,
    version: String,
    flavor: String,
    status: InstallStatusDto,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStatusDto {
    Installing,
    Installed { installation: InstallationDto },
    Failed { error: ErrorDto },
}

#[derive(Serialize, Debug)]
pub struct InstallationDto {
    dir: String,
}

#[derive(Serialize, Debug)]
pub struct InstallUpdateEventArgsDto {
    id: String,
    status: InstallStatusDto,
}

#[derive(Serialize, Debug)]
pub struct InstallRemoveEventArgsDto {
    id: String,
}

impl<E> From<E> for ErrorDto
where
    E: Borrow<Error>,
{
    fn from(value: E) -> Self {
        let value = value.borrow();
        Self {
            message: value.to_string(),
        }
    }
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

impl From<Install> for InstallDto {
    fn from(value: Install) -> Self {
        Self {
            id: value.id,
            version: value.version,
            flavor: value.flavor,
            status: value.status.into(),
        }
    }
}

impl<I> From<I> for InstallStatusDto
where
    I: Borrow<InstallStatus>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        match value {
            InstallStatus::Installing => Self::Installing,
            InstallStatus::Installed(installation) => Self::Installed {
                installation: installation.as_ref().into(),
            },
            InstallStatus::Failed(e) => Self::Failed {
                error: e.as_ref().into(),
            },
        }
    }
}

impl<I> From<I> for InstallationDto
where
    I: Borrow<Installation>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            dir: value.dir.to_string_lossy().into_owned(),
        }
    }
}

impl<I> From<I> for InstallUpdateEventArgsDto
where
    I: Borrow<InstallUpdateEventArgs>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            id: value.id.clone(),
            status: InstallStatusDto::from(&value.status),
        }
    }
}

impl<I> From<I> for InstallRemoveEventArgsDto
where
    I: Borrow<InstallRemoveEventArgs>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            id: value.id.clone(),
        }
    }
}
