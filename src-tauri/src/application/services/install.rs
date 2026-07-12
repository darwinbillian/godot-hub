use std::{
    borrow::Borrow,
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio_stream::StreamExt;

use crate::application::{
    error::Error,
    event::Event,
    services::{
        download::{DownloadProgress, DownloadRequest, DownloadService, DownloadStatus},
        installation::{
            Installation, InstallationMetadata, InstallationRemoveEventArgs, InstallationService,
        },
        task::{Task, TaskReporter, TaskService, TaskStartEventArgs, TaskStatus},
    },
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
}

pub struct InstallServiceInner {
    download_service: DownloadService,
    installation_service: InstallationService,
    task_service: TaskService<InstallState, InstallProgress, Installation>,
    add_event: Event<InstallAddEventArgs>,
    update_event: Event<InstallUpdateEventArgs>,
    remove_event: Event<InstallRemoveEventArgs>,
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

pub struct InstallAddEventArgs;

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
        installation_service: InstallationService,
        task_service: TaskService<InstallState, InstallProgress, Installation>,
    ) -> Self {
        let add_event = Event::new();
        let update_event = Event::new();
        let remove_event = Event::new();

        installation_service
            .remove_event()
            .map(InstallRemoveEventArgs::from)
            .subscribe(remove_event.clone());

        task_service
            .start_event()
            .map(InstallAddEventArgs::from)
            .subscribe(add_event.clone());

        task_service
            .update_event()
            .filter_map(|args| {
                let status = match &args.status {
                    TaskStatus::Pending => return None,
                    TaskStatus::Completed(installation) => {
                        InstallStatus::Installed(installation.clone())
                    }
                    TaskStatus::Running(progress) => InstallStatus::Installing(progress.clone()),
                    TaskStatus::Failed(e) => InstallStatus::Failed(e.clone()),
                };

                let args = InstallUpdateEventArgs {
                    id: args.state.id.clone(),
                    version: args.state.version.clone(),
                    flavor: args.state.flavor.clone(),
                    status,
                };

                Some(args)
            })
            .subscribe(update_event.clone());

        Self {
            inner: Arc::new(InstallServiceInner {
                download_service,
                installation_service,
                task_service,
                add_event,
                update_event,
                remove_event,
            }),
        }
    }

    pub fn add_event(&self) -> &Event<InstallAddEventArgs> {
        &self.inner.add_event
    }

    pub fn update_event(&self) -> &Event<InstallUpdateEventArgs> {
        &self.inner.update_event
    }

    pub fn remove_event(&self) -> &Event<InstallRemoveEventArgs> {
        &self.inner.remove_event
    }

    pub async fn install(&self, version: &str, flavor: &str) -> Result<(), Error> {
        let id = format!("{}-{}", version, flavor);
        let state = InstallState::new(&id, version, flavor);
        let task = Task::new(&id, state);

        self.inner
            .task_service
            .run(task, async |reporter| -> Result<Installation, Error> {
                let download_path = self.download(reporter.clone(), version, flavor).await?;

                reporter.report(InstallProgress::Extracting);
                let installation = self.inner.installation_service.create(&id, version, flavor);
                crate::application::utils::zip::extract(download_path, &installation.dir).await?;

                reporter.report(InstallProgress::Finalizing);
                let executable = format!("Godot_v{}-{}_win64.exe", version, flavor);
                let metadata = InstallationMetadata {
                    version: version.to_owned(),
                    flavor: flavor.to_owned(),
                    executable,
                };

                metadata.save(&installation.dir).await?;

                Ok(installation)
            })
            .await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Install>, Error> {
        let mut installs = HashMap::<String, Install>::new();

        let installations = self.inner.installation_service.list().await?;
        let tasks = self.inner.task_service.list();

        for task in tasks {
            let status = match task.status {
                TaskStatus::Pending => continue,
                TaskStatus::Running(progress) => InstallStatus::Installing(progress),
                TaskStatus::Completed(_) => continue,
                TaskStatus::Failed(e) => InstallStatus::Failed(e),
            };

            let install = Install {
                id: task.state.id.clone(),
                version: task.state.version.clone(),
                flavor: task.state.flavor.clone(),
                status,
            };

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
}

impl InstallState {
    pub fn new(id: &str, version: &str, flavor: &str) -> Self {
        Self {
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
        }
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

impl<T> From<T> for InstallAddEventArgs
where
    T: Borrow<TaskStartEventArgs>,
{
    fn from(_value: T) -> Self {
        Self
    }
}

impl<I> From<I> for InstallRemoveEventArgs
where
    I: Borrow<InstallationRemoveEventArgs>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            id: value.id.clone(),
        }
    }
}
