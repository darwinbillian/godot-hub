use std::{collections::HashMap, sync::Arc};

use crate::application::{
    error::Error,
    services::{
        installation::{Installation, InstallationService},
        installer::{InstallerProgress, InstallerService, InstallerState},
        task::{Task, TaskService, TaskStatus},
    },
    utils::event::Event,
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
}

#[derive(Clone)]
pub struct Install {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: InstallStatus,
}

#[derive(Clone)]
pub enum InstallStatus {
    Installing(Arc<InstallerProgress>),
    Paused(Arc<InstallerProgress>),
    Installed(Arc<Installation>),
    Failed(Arc<Error>),
}

pub struct InstallAddEventArgs;

pub struct InstallUpdateEventArgs {
    pub id: String,
    pub status: InstallStatus,
}

pub struct InstallRemoveEventArgs {
    pub id: String,
}

struct InstallServiceInner {
    installation_service: InstallationService,
    installer_service: InstallerService,
    task_service: TaskService<InstallerState, InstallerProgress, Installation>,
    add_event: Event<InstallAddEventArgs>,
    update_event: Event<InstallUpdateEventArgs>,
    remove_event: Event<InstallRemoveEventArgs>,
}

impl InstallService {
    pub fn new(
        installation_service: InstallationService,
        installer_service: InstallerService,
        task_service: TaskService<InstallerState, InstallerProgress, Installation>,
    ) -> Self {
        let add_event = Event::new();
        let update_event = Event::new();
        let remove_event = Event::new();

        installation_service
            .remove_event()
            .map(|args| InstallRemoveEventArgs::new(&args.id))
            .subscribe(remove_event.clone());

        task_service
            .start_event()
            .map(|_args| InstallAddEventArgs::new())
            .subscribe(add_event.clone());

        task_service
            .update_event()
            .filter_map(|args| {
                let status = match &args.status {
                    TaskStatus::Running(progress) => InstallStatus::Installing(progress.clone()),
                    TaskStatus::Paused(progress) => InstallStatus::Paused(progress.clone()),
                    TaskStatus::Completed(installation) => {
                        InstallStatus::Installed(installation.clone())
                    }
                    TaskStatus::Failed(e) => InstallStatus::Failed(e.clone()),
                    _ => return None,
                };

                let args = InstallUpdateEventArgs::new(&args.state.id, status);
                Some(args)
            })
            .subscribe(update_event.clone());

        task_service
            .update_event()
            .filter_map(|args| {
                let args = match &args.status {
                    TaskStatus::Cancelled => InstallRemoveEventArgs::new(&args.state.id),
                    _ => return None,
                };

                Some(args)
            })
            .subscribe(remove_event.clone());

        Self {
            inner: Arc::new(InstallServiceInner {
                installation_service,
                installer_service,
                task_service,
                add_event,
                update_event,
                remove_event,
            }),
        }
    }

    pub fn task_service(&self) -> &TaskService<InstallerState, InstallerProgress, Installation> {
        &self.inner.task_service
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
        let installer = self.inner.installer_service.create(version, flavor);
        let state = InstallerState::from(&installer);
        let task = Task::new(&state.id.clone(), state);

        self.inner.task_service.run(task, async move |controller| {
            let installation = installer.install(&controller).await?;
            Ok(installation)
        });

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Install>, Error> {
        let mut installs = HashMap::<String, Install>::new();

        let installations = self.inner.installation_service.list().await?;
        let tasks = self.inner.task_service.list();

        for task in tasks {
            let status = match task.status {
                TaskStatus::Paused(progress) => InstallStatus::Paused(progress),
                TaskStatus::Running(progress) => InstallStatus::Installing(progress),
                TaskStatus::Failed(e) => InstallStatus::Failed(e),
                _ => continue,
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
            let install = Install {
                id: installation.id.clone(),
                flavor: installation.flavor.clone(),
                version: installation.version.clone(),
                status: InstallStatus::Installed(Arc::new(installation)),
            };

            installs.insert(install.id.clone(), install);
        }

        let mut installs = installs.into_values().collect::<Vec<Install>>();
        installs.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        Ok(installs)
    }
}

impl InstallAddEventArgs {
    pub fn new() -> Self {
        Self
    }
}

impl InstallUpdateEventArgs {
    pub fn new(id: &str, status: InstallStatus) -> Self {
        Self {
            id: id.to_owned(),
            status: status.clone(),
        }
    }
}

impl InstallRemoveEventArgs {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}
