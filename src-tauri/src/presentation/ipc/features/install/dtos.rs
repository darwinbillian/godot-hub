use std::borrow::Borrow;

use serde::Serialize;

use crate::{
    application::services::{
        download::DownloadProgress,
        install::{
            Install, InstallProgress, InstallRemoveEventArgs, InstallStatus,
            InstallUpdateEventArgs, Installation,
        },
    },
    presentation::ipc::dtos::ErrorDto,
};

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
    Installing { progress: InstallProgressDto },
    Installed { installation: InstallationDto },
    Failed { error: ErrorDto },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallProgressDto {
    Starting,
    Downloading { progress: DownloadProgressDto },
    Extracting,
    Finalizing,
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

#[derive(Serialize, Debug)]
pub struct DownloadProgressDto {
    downloaded: u64,
    size: Option<u64>,
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
            InstallStatus::Installing(progress) => Self::Installing {
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

impl<I> From<I> for InstallProgressDto
where
    I: Borrow<InstallProgress>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        match value {
            InstallProgress::Starting => Self::Starting,
            InstallProgress::Downloading(progress) => Self::Downloading {
                progress: progress.into(),
            },
            InstallProgress::Extracting => Self::Extracting,
            InstallProgress::Finalizing => Self::Finalizing,
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
