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
        Self {
            message: value.borrow().to_string(),
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

impl From<VersionStatus> for VersionStatusDto {
    fn from(value: VersionStatus) -> Self {
        match value {
            VersionStatus::Available => VersionStatusDto::Available,
            VersionStatus::Installing => VersionStatusDto::Installing,
            VersionStatus::Installed => VersionStatusDto::Installed,
            VersionStatus::Failed(e) => VersionStatusDto::Failed { error: e.into() },
        }
    }
}

impl From<VersionUpdateEventArgs> for VersionUpdateEventArgsDto {
    fn from(value: VersionUpdateEventArgs) -> Self {
        Self {
            name: value.name,
            flavor: value.flavor,
            status: value.status.into(),
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

impl From<InstallStatus> for InstallStatusDto {
    fn from(value: InstallStatus) -> Self {
        match value {
            InstallStatus::Installing => InstallStatusDto::Installing,
            InstallStatus::Installed(installation) => InstallStatusDto::Installed {
                installation: installation.into(),
            },
            InstallStatus::Failed(e) => InstallStatusDto::Failed { error: e.into() },
        }
    }
}

impl<I> From<I> for InstallationDto
where
    I: Borrow<Installation>,
{
    fn from(value: I) -> Self {
        Self {
            dir: value.borrow().dir.to_string_lossy().into_owned(),
        }
    }
}

impl From<InstallUpdateEventArgs> for InstallUpdateEventArgsDto {
    fn from(value: InstallUpdateEventArgs) -> Self {
        Self {
            id: value.id,
            status: value.status.into(),
        }
    }
}

impl From<InstallRemoveEventArgs> for InstallRemoveEventArgsDto {
    fn from(value: InstallRemoveEventArgs) -> Self {
        Self { id: value.id }
    }
}
