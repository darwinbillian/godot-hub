use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::{
        download::DownloadProgress,
        install::{
            Install, InstallAddEventArgs, InstallRemoveEventArgs, InstallStatus,
            InstallUpdateEventArgs,
        },
        installation::Installation,
        installer::InstallerProgress,
    },
    presentation::ipc::dtos::ErrorDto,
};

#[derive(Serialize, Debug)]
pub struct InstallDto {
    id: String,
    name: String,
    version: String,
    flavor: String,
    status: InstallStatusDto,
}

impl From<Install> for InstallDto {
    fn from(value: Install) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version,
            flavor: value.flavor,
            status: value.status.into(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStatusDto {
    Installing { progress: InstallerProgressDto },
    Paused { progress: InstallerProgressDto },
    Installed { installation: InstallationDto },
    Failed { error: ErrorDto },
}

impl<I> From<I> for InstallStatusDto
where
    I: Borrow<InstallStatus>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        match value {
            InstallStatus::Installing(progress) => Self::Installing {
                progress: progress.as_ref().into(),
            },
            InstallStatus::Paused(progress) => Self::Paused {
                progress: progress.as_ref().into(),
            },
            InstallStatus::Installed(installation) => Self::Installed {
                installation: installation.as_ref().into(),
            },
            InstallStatus::Failed(e) => Self::Failed {
                error: e.as_ref().into(),
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct InstallAddEventArgsDto;

impl<I> From<I> for InstallAddEventArgsDto
where
    I: Borrow<InstallAddEventArgs>,
{
    fn from(_value: I) -> Self {
        Self
    }
}

#[derive(Serialize, Debug)]
pub struct InstallUpdateEventArgsDto {
    id: String,
    status: InstallStatusDto,
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

#[derive(Serialize, Debug)]
pub struct InstallRemoveEventArgsDto {
    id: String,
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

#[derive(Serialize, Debug)]
pub struct DownloadProgressDto {
    downloaded: u64,
    size: Option<u64>,
}

impl<D> From<D> for DownloadProgressDto
where
    D: Borrow<DownloadProgress>,
{
    fn from(value: D) -> Self {
        let value = value.borrow();
        Self {
            downloaded: value.downloaded,
            size: value.size,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct InstallationDto {
    dir: String,
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

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallerProgressDto {
    Starting,
    Downloading { progress: DownloadProgressDto },
    Extracting,
    Finalizing,
}

impl<I> From<I> for InstallerProgressDto
where
    I: Borrow<InstallerProgress>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        match value {
            InstallerProgress::Starting => Self::Starting,
            InstallerProgress::Downloading(progress) => Self::Downloading {
                progress: progress.into(),
            },
            InstallerProgress::Extracting => Self::Extracting,
            InstallerProgress::Finalizing => Self::Finalizing,
        }
    }
}
