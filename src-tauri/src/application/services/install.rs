use std::{
    borrow::Borrow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio_stream::StreamExt;

use crate::application::{
    error::Error,
    event::{EventAdapter, EventDispatcher, EventRepeater},
    services::{
        download::{DownloadProgress, DownloadRequest, DownloadService, DownloadStatus},
        task::{Task, TaskReporter, TaskService, TaskStatus, TaskUpdateEventArgs},
    },
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
}

pub struct InstallServiceInner {
    download_service: DownloadService,
    task_service: TaskService<InstallState, InstallProgress, Installation>,
    update_event: EventAdapter<InstallUpdateEventArgs>,
    remove_event: EventRepeater<InstallRemoveEventArgs>,
    dir: PathBuf,
}

pub struct InstallState {
    pub id: String,
    pub version: String,
    pub flavor: String,
}

pub struct Install {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: InstallStatus,
}

#[derive(Clone)]
pub enum InstallStatus {
    Installing(Arc<InstallProgress>),
    Installed(Arc<Installation>),
    Failed(Arc<Error>),
}

#[derive(Default)]
pub enum InstallProgress {
    #[default]
    Starting,
    Downloading(DownloadProgress),
    Extracting,
    Finalizing,
}

pub struct Installation {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub dir: PathBuf,
}

pub struct InstallationHandle {
    remove_event: EventDispatcher<InstallRemoveEventArgs>,
    id: String,
    dir: PathBuf,
    executable: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallationMetadata {
    pub version: String,
    pub flavor: String,
    pub executable: String,
}

pub struct InstallUpdateEventArgs {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: InstallStatus,
}

pub struct InstallRemoveEventArgs {
    pub id: String,
}

impl InstallService {
    pub fn new(
        download_service: DownloadService,
        task_service: TaskService<InstallState, InstallProgress, Installation>,
        dir: PathBuf,
    ) -> Self {
        let update_event = EventAdapter::new(task_service.update_event());
        Self {
            inner: Arc::new(InstallServiceInner {
                download_service,
                task_service,
                update_event,
                remove_event: EventRepeater::new(),
                dir,
            }),
        }
    }

    pub fn update_event(&self) -> &EventAdapter<InstallUpdateEventArgs> {
        &self.inner.update_event
    }

    pub fn remove_event(&self) -> &EventRepeater<InstallRemoveEventArgs> {
        &self.inner.remove_event
    }

    pub async fn install(&self, version: &str, flavor: &str) -> Result<(), Error> {
        let id = format!("{}-{}", version, flavor);
        let state = InstallState {
            id: id.clone(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
        };
        let task = Task::new(&id, state);

        self.inner
            .task_service
            .run(task, async |reporter| -> Result<Installation, Error> {
                let download_path = self.download(reporter.clone(), version, flavor).await?;

                reporter.report(InstallProgress::Extracting);
                let dir = self.inner.dir.join(&id);
                crate::application::utils::zip::extract(download_path, &dir).await?;

                reporter.report(InstallProgress::Finalizing);
                let executable = format!("Godot_v{}-{}_win64.exe", version, flavor);
                let metadata = InstallationMetadata {
                    version: version.to_owned(),
                    flavor: flavor.to_owned(),
                    executable,
                };
                metadata.save(&dir).await?;

                let installation = Installation {
                    id,
                    version: metadata.version,
                    flavor: metadata.flavor,
                    dir,
                };
                Ok(installation)
            })
            .await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Install>, Error> {
        let mut installs = HashMap::<String, Install>::new();

        let installations = self.list_installations().await?;
        let tasks = self.inner.task_service.list();

        for task in tasks {
            let install = Install::from(task);
            installs.insert(install.id.clone(), install);
        }

        for installation in installations {
            let install = Install::from(installation);
            installs.insert(install.id.clone(), install);
        }

        let mut installs = installs.into_values().collect::<Vec<Install>>();
        installs.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        Ok(installs)
    }

    pub async fn get(&self, id: &str) -> Result<InstallationHandle, Error> {
        let dir = self.inner.dir.join(id);
        let metadata = InstallationMetadata::load(&dir).await?;
        let install = InstallationHandle::new(id, &dir, &metadata.executable);
        self.inner.remove_event.repeat(install.remove_event());
        Ok(install)
    }

    async fn download(
        &self,
        reporter: TaskReporter<InstallState, InstallProgress, Installation>,
        version: &str,
        flavor: &str,
    ) -> Result<PathBuf, Error> {
        let request = DownloadRequest::new(version, flavor, "win64.exe.zip", "windows.64");
        let mut handle = self.inner.download_service.download(request).await?;

        let mut last_progress = Instant::now();

        while let Some(progress) = handle.stream.try_next().await? {
            if progress.status != DownloadStatus::Downloading
                || last_progress.elapsed() > Duration::from_millis(500)
            {
                reporter.report(InstallProgress::Downloading(progress));
                last_progress = Instant::now();
            }
        }

        Ok(handle.path)
    }

    async fn list_installations(&self) -> Result<Vec<Installation>, Error> {
        let mut installations = Vec::<Installation>::new();

        let mut entries = match tokio::fs::read_dir(&self.inner.dir).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(installations),
            entries => entries?,
        };

        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if !file_type.is_dir() {
                continue;
            }

            let id = match entry.file_name().into_string() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let dir = entry.path();
            let metadata = match InstallationMetadata::load(&dir).await {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            let installation = Installation {
                id,
                version: metadata.version,
                flavor: metadata.flavor,
                dir,
            };
            installations.push(installation);
        }

        Ok(installations)
    }
}

impl InstallationHandle {
    pub fn new(id: &str, dir: &Path, executable: &str) -> Self {
        Self {
            remove_event: EventDispatcher::new(),
            id: id.to_owned(),
            dir: dir.to_owned(),
            executable: dir.join(executable),
        }
    }

    pub fn remove_event(&self) -> &EventDispatcher<InstallRemoveEventArgs> {
        &self.remove_event
    }

    pub async fn launch(&self) -> Result<(), Error> {
        Command::new(&self.executable).spawn()?;
        Ok(())
    }

    pub async fn uninstall(&self) -> Result<(), Error> {
        tokio::fs::remove_dir_all(&self.dir).await?;

        let args = InstallRemoveEventArgs {
            id: self.id.clone(),
        };
        self.remove_event.invoke(Arc::new(args));

        Ok(())
    }

    pub async fn reveal(&self) -> Result<(), Error> {
        tauri_plugin_opener::reveal_item_in_dir(&self.executable)?;
        Ok(())
    }
}

impl InstallationMetadata {
    pub async fn save(&self, dir: &Path) -> Result<(), Error> {
        let bytes = serde_json::to_vec(self)?;
        let path = dir.join("metadata.hub.json");
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    pub async fn load(dir: &Path) -> Result<InstallationMetadata, Error> {
        let path = dir.join("metadata.hub.json");
        let bytes = tokio::fs::read(path).await?;
        let metadata = serde_json::from_slice::<InstallationMetadata>(&bytes)?;
        Ok(metadata)
    }
}

impl From<Installation> for Install {
    fn from(value: Installation) -> Self {
        Self {
            id: value.id.clone(),
            version: value.version.clone(),
            flavor: value.flavor.clone(),
            status: InstallStatus::Installed(Arc::new(value)),
        }
    }
}

impl From<Task<InstallState, InstallProgress, Installation>> for Install {
    fn from(value: Task<InstallState, InstallProgress, Installation>) -> Self {
        Install {
            id: value.state.id.clone(),
            flavor: value.state.flavor.clone(),
            version: value.state.version.clone(),
            status: value.status.into(),
        }
    }
}

impl<T> From<T> for InstallStatus
where
    T: Borrow<TaskStatus<InstallProgress, Installation>>,
{
    fn from(value: T) -> Self {
        match value.borrow() {
            TaskStatus::Pending => Self::Installing(Arc::new(InstallProgress::default())),
            TaskStatus::Running(progress) => Self::Installing(progress.clone()),
            TaskStatus::Completed(installation) => Self::Installed(installation.clone()),
            TaskStatus::Failed(e) => Self::Failed(e.clone()),
        }
    }
}

impl<T> From<T> for InstallUpdateEventArgs
where
    T: Borrow<TaskUpdateEventArgs<InstallState, InstallProgress, Installation>>,
{
    fn from(value: T) -> Self {
        let value = value.borrow();
        Self {
            id: value.state.id.clone(),
            version: value.state.version.clone(),
            flavor: value.state.flavor.clone(),
            status: InstallStatus::from(&value.status),
        }
    }
}
