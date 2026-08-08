use std::{cmp::Reverse, sync::Arc};

use anyhow::{Error, Result};
use itertools::Itertools;

use crate::{
    application::{
        services::{
            installation::{Installation, InstallationService},
            installer::{InstallerProgress, InstallerService, InstallerState},
            task::{Task, TaskService, TaskStatus},
        },
        utils::event::Event,
    },
    domain::models::version::{Flavor, FlavorKindFlags, Variant, Version, VersionFlavorVariant},
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
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

    pub async fn install(&self, version: Version, flavor: Flavor, variant: Variant) -> Result<()> {
        let installer = self
            .inner
            .installer_service
            .create(version, flavor, variant);
        let state = InstallerState::from(&installer);
        let task = Task::new(&state.id.clone(), state);

        self.inner.task_service.run(task, async move |controller| {
            let installation = installer.install(&controller).await?;
            Ok(installation)
        });

        Ok(())
    }

    pub async fn list(&self, filter: Option<InstallFilter>) -> Result<Vec<Install>> {
        let installations = self
            .inner
            .installation_service
            .list()
            .await?
            .into_iter()
            .map(|installation| Install {
                id: installation.id.clone(),
                name: installation.name.clone(),
                version: installation.version,
                flavor: installation.flavor,
                variant: installation.variant,
                status: InstallStatus::Installed(Arc::new(installation)),
            });

        let tasks = self
            .inner
            .task_service
            .list()
            .into_iter()
            .filter_map(|task| {
                let status = match task.status {
                    TaskStatus::Paused(progress) => InstallStatus::Paused(progress),
                    TaskStatus::Running(progress) => InstallStatus::Installing(progress),
                    TaskStatus::Failed(e) => InstallStatus::Failed(e),
                    _ => return None,
                };

                let install = Install {
                    id: task.state.id.clone(),
                    name: task.state.name.clone(),
                    version: task.state.version,
                    flavor: task.state.flavor,
                    variant: task.state.variant,
                    status,
                };

                Some(install)
            });

        let installs = installations
            .chain(tasks)
            .filter(|install| {
                filter
                    .as_ref()
                    .and_then(|filter| filter.flavor)
                    .unwrap_or_default()
                    .contains(install.flavor)
            })
            .unique_by(|install| install.id.clone())
            .sorted_unstable_by_key(|install| {
                Reverse(VersionFlavorVariant::new(
                    install.version,
                    install.flavor,
                    install.variant,
                ))
            })
            .collect::<Vec<Install>>();

        Ok(installs)
    }
}

#[derive(Clone)]
pub struct Install {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub flavor: Flavor,
    pub variant: Variant,
    pub status: InstallStatus,
}

#[derive(Clone)]
pub enum InstallStatus {
    Installing(Arc<InstallerProgress>),
    Paused(Arc<InstallerProgress>),
    Installed(Arc<Installation>),
    Failed(Arc<Error>),
}

pub struct InstallFilter {
    pub flavor: Option<FlavorKindFlags>,
}

pub struct InstallAddEventArgs;

impl InstallAddEventArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct InstallUpdateEventArgs {
    pub id: String,
    pub status: InstallStatus,
}

impl InstallUpdateEventArgs {
    pub fn new(id: &str, status: InstallStatus) -> Self {
        Self {
            id: id.to_owned(),
            status: status.clone(),
        }
    }
}

pub struct InstallRemoveEventArgs {
    pub id: String,
}

impl InstallRemoveEventArgs {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}
