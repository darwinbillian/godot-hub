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
    interfaces::{download::DownloadRequest, download_configs::DownloadConfigsProvider},
    services::{
        download::{DownloadProgress, DownloadService, DownloadStatus},
        installation::{Installation, InstallationService, InstallationTransaction},
        platform::PlatformService,
        task::{TaskController, TaskError},
    },
    utils::{fs::DirectoryGuard, zip::ZipFile},
};

pub struct InstallerService {
    inner: Arc<InstallerServiceInner>,
}

struct InstallerServiceInner {
    download_configs_provider: Arc<dyn DownloadConfigsProvider + Send + Sync>,
    download_service: DownloadService,
    installation_service: InstallationService,
    platform_service: PlatformService,
}

impl InstallerService {
    pub fn new(
        download_configs_provider: Arc<dyn DownloadConfigsProvider + Send + Sync>,
        download_service: DownloadService,
        installation_service: InstallationService,
        platform_service: PlatformService,
    ) -> Self {
        Self {
            inner: Arc::new(InstallerServiceInner {
                download_configs_provider,
                download_service,
                installation_service,
                platform_service,
            }),
        }
    }

    pub fn create(&self, version: &str, flavor: &str, mono: bool) -> Installer {
        let id = format!("{}-{}{}", version, flavor, if mono { "-mono" } else { "" });
        let name = format!("Godot {}{}", version, if mono { " Mono" } else { "" });
        Installer {
            download_configs_provider: self.inner.download_configs_provider.clone(),
            download_service: self.inner.download_service.clone(),
            installation_service: self.inner.installation_service.clone(),
            platform_service: self.inner.platform_service.clone(),
            id,
            name,
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            mono,
        }
    }
}

pub struct Installer {
    download_configs_provider: Arc<dyn DownloadConfigsProvider + Send + Sync>,
    download_service: DownloadService,
    installation_service: InstallationService,
    platform_service: PlatformService,
    id: String,
    name: String,
    version: String,
    flavor: String,
    mono: bool,
}

impl Installer {
    pub async fn install(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
    ) -> Result<Installation, TaskError> {
        let platform = self.platform_service.get_platform()?;
        let slug = self.get_slug(&platform).await?;

        let transaction = self.installation_service.create(
            &self.id,
            &self.name,
            &self.version,
            &self.flavor,
            self.mono,
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
    ) -> Result<PathBuf> {
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
        archive
            .extract_unwrapped_root_dir(transaction.dir())
            .await?;
        Ok(())
    }

    async fn finalize(
        &self,
        controller: &TaskController<InstallerState, InstallerProgress, Installation>,
        transaction: InstallationTransaction,
        executable: &Path,
    ) -> Result<Installation> {
        controller.report(InstallerProgress::Finalizing);
        let installation = transaction.commit(executable).await?;
        Ok(installation)
    }

    async fn get_slug(&self, platform: &str) -> Result<String> {
        let download_configs = self
            .download_configs_provider
            .get_download_configs()
            .await?;
        let slug = download_configs.get_slug(&self.version, &self.flavor, self.mono, platform)?;
        Ok(slug)
    }

    async fn find_executable(&self, slug: &str, download_path: &Path) -> Result<PathBuf> {
        let archive = ZipFile::open(download_path).await?;
        let root_dir = archive.root_dir()?;
        let executable = archive
            .file_names()
            .into_iter()
            .max_by_key(|file_name| {
                let file_name = file_name.rsplit("/").next().unwrap();

                let mut score = 0;
                score += file_name.contains("Godot") as i32;
                score += file_name.contains(&self.version) as i32;
                score += file_name.contains(&self.flavor) as i32;
                score += file_name.contains(slug.strip_suffix(".zip").unwrap_or(slug)) as i32 * 5;
                score -= file_name.contains("console") as i32;
                score
            })
            .map(|executable| {
                let executable = PathBuf::from(executable);

                if let Some(root_dir) = &root_dir {
                    executable.strip_prefix(root_dir).unwrap().to_owned()
                } else {
                    executable
                }
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
    pub mono: bool,
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
            mono: value.mono,
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
    #[error("executable not found")]
    ExecutableNotFound,
}
