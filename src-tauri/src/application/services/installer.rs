use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use thiserror::Error;
use tokio_stream::StreamExt;

use crate::application::{
    services::{
        download::{DownloadProgress, DownloadRequest, DownloadService, DownloadStatus},
        installation::{Installation, InstallationService, InstallationTransaction},
        task::{TaskController, TaskError},
    },
    utils::{fs::DirectoryGuard, zip::ZipFile},
};

pub struct InstallerService {
    inner: Arc<InstallerServiceInner>,
}

struct InstallerServiceInner {
    download_service: DownloadService,
    installation_service: InstallationService,
}

impl InstallerService {
    pub fn new(
        download_service: DownloadService,
        installation_service: InstallationService,
    ) -> Self {
        Self {
            inner: Arc::new(InstallerServiceInner {
                download_service,
                installation_service,
            }),
        }
    }

    pub fn create(&self, version: &str, flavor: &str) -> Installer {
        let id = format!("{}-{}", version, flavor);
        let name = format!("Godot {}", version);
        Installer {
            download_service: self.inner.download_service.clone(),
            installation_service: self.inner.installation_service.clone(),
            id,
            name,
            version: version.to_owned(),
            flavor: flavor.to_owned(),
        }
    }
}

pub struct Installer {
    download_service: DownloadService,
    installation_service: InstallationService,
    id: String,
    name: String,
    version: String,
    flavor: String,
}

impl Installer {
    pub async fn install(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
    ) -> Result<Installation, TaskError> {
        let (slug, platform) = self.get_slug_and_platform()?;

        let transaction = self.installation_service.create(
            &self.id,
            &self.name,
            &self.version,
            &self.flavor,
            &platform,
        );

        let mut dir = DirectoryGuard::create(transaction.dir()).await?;

        let download_path = self.download(controller, &slug, &platform).await?;
        let executable = self.verify(controller, &slug, &download_path).await?;
        self.extract(controller, &transaction, &download_path)
            .await?;
        let installation = self.finalize(controller, transaction, &executable).await?;

        dir.disarm();

        Ok(installation)
    }

    async fn download(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        slug: &str,
        platform: &str,
    ) -> Result<PathBuf, TaskError> {
        let request = DownloadRequest::new(&self.version, &self.flavor, slug, platform);
        let mut handle = self
            .download_service
            .download(request, controller.cancellation_token().clone())
            .await?;

        let mut last_progress = Instant::now();

        while let Some(progress) = handle.stream.try_next().await? {
            if progress.status != DownloadStatus::Downloading
                || last_progress.elapsed() > Duration::from_millis(500)
            {
                controller.report(InstallerProgress::Downloading(progress));
                last_progress = Instant::now();
            }

            controller.cancelled_or_paused().await?;
        }

        Ok(handle.path)
    }

    async fn verify(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        slug: &str,
        download_path: &Path,
    ) -> Result<String> {
        controller.report(InstallerProgress::Verifying);
        let executable = self.find_executable(slug, download_path).await?;
        Ok(executable)
    }

    async fn extract(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        transaction: &InstallationTransaction,
        download_path: &Path,
    ) -> Result<()> {
        controller.report(InstallerProgress::Extracting);
        let archive = ZipFile::open(download_path).await?;
        archive.extract(transaction.dir()).await?;
        Ok(())
    }

    async fn finalize(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        transaction: InstallationTransaction,
        executable: &str,
    ) -> Result<Installation> {
        controller.report(InstallerProgress::Finalizing);
        let installation = transaction.commit(executable).await?;
        Ok(installation)
    }

    fn get_slug_and_platform(&self) -> Result<(String, String)> {
        let (slug, platform) = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => ("win64.exe.zip", "windows.64"),
            (os, arch) => {
                return Err(anyhow::anyhow!(InstallerError::PlatformNotSupported {
                    arch: arch.to_owned(),
                    os: os.to_owned(),
                }))
            }
        };

        Ok((slug.to_owned(), platform.to_owned()))
    }

    async fn find_executable(&self, slug: &str, download_path: &Path) -> Result<String> {
        let archive = ZipFile::open(download_path).await?;

        let executable = archive
            .file_names()
            .into_iter()
            .max_by_key(|file_name| {
                let mut score = 0;
                if file_name.contains("Godot") {
                    score += 1;
                }
                if file_name.contains(&self.version) {
                    score += 1;
                }
                if file_name.contains(&self.flavor) {
                    score += 1;
                }
                if file_name.contains(slug.strip_suffix(".zip").unwrap_or(slug)) {
                    score += 5;
                }
                if file_name.contains("console") {
                    score -= 1;
                }
                score
            })
            .ok_or(InstallerError::ExecutableNotFound)?;

        Ok(executable)
    }
}

pub struct InstallerState {
    pub id: String,
    pub name: String,
    pub version: String,
    pub flavor: String,
}

impl<I> From<I> for InstallerState
where
    I: Borrow<Installer>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            version: value.version.clone(),
            flavor: value.flavor.clone(),
        }
    }
}

#[derive(Default)]
pub enum InstallerProgress {
    #[default]
    Starting,
    Downloading(DownloadProgress),
    Verifying,
    Extracting,
    Finalizing,
}

#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("platform '{os}-{arch}' is not supported")]
    PlatformNotSupported { arch: String, os: String },
    #[error("executable not found")]
    ExecutableNotFound,
}
