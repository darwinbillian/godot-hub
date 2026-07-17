use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio_stream::StreamExt;

use crate::application::services::{
    download::{DownloadProgress, DownloadRequest, DownloadService, DownloadStatus},
    installation::{Installation, InstallationService, InstallationTransaction},
    task::{TaskController, TaskError},
};

pub struct InstallerService {
    inner: Arc<InstallerServiceInner>,
}

pub struct Installer {
    download_service: DownloadService,
    installation_service: InstallationService,
    id: String,
    version: String,
    flavor: String,
}

pub struct InstallerState {
    pub id: String,
    pub version: String,
    pub flavor: String,
}

#[derive(Default)]
pub enum InstallerProgress {
    #[default]
    Starting,
    Downloading(DownloadProgress),
    Extracting,
    Finalizing,
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
        Installer {
            download_service: self.inner.download_service.clone(),
            installation_service: self.inner.installation_service.clone(),
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
        }
    }
}

impl Installer {
    pub async fn install(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
    ) -> Result<Installation, TaskError> {
        let transaction = self
            .installation_service
            .create(&self.id, &self.version, &self.flavor);
        let download_path = self.download(controller).await?;
        self.extract(controller, &transaction, &download_path)
            .await?;
        let installation = self.finalize(controller, transaction).await?;
        Ok(installation)
    }

    async fn download(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
    ) -> Result<PathBuf, TaskError> {
        let request =
            DownloadRequest::new(&self.version, &self.flavor, "win64.exe.zip", "windows.64");

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

    async fn extract(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        transaction: &InstallationTransaction,
        download_path: &Path,
    ) -> Result<()> {
        controller.report(InstallerProgress::Extracting);
        crate::application::utils::zip::extract(download_path, &transaction.dir()).await?;
        Ok(())
    }

    async fn finalize(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        transaction: InstallationTransaction,
    ) -> Result<Installation> {
        controller.report(InstallerProgress::Finalizing);
        let executable = format!("Godot_v{}-{}_win64.exe", self.version, self.flavor);
        let installation = transaction.commit(&executable).await?;
        Ok(installation)
    }
}

impl<I> From<I> for InstallerState
where
    I: Borrow<Installer>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            id: value.id.clone(),
            version: value.version.clone(),
            flavor: value.flavor.clone(),
        }
    }
}
