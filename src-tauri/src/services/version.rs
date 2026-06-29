use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use reqwest_middleware::ClientWithMiddleware;

use crate::{
    error::Error,
    services::{
        install::InstallService,
        task::{Task, TaskService, TaskStatus},
    },
};

pub struct VersionService {
    client: ClientWithMiddleware,
    install_service: InstallService,
    task_service: TaskService,
    update_event: VersionUpdateEvent,
}

pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: VersionStatus,
}

pub enum VersionStatus {
    Available,
    Installing,
    Installed,
    Failed(Arc<Error>),
}

pub struct VersionUpdateEvent {
    task_service: TaskService,
}

pub struct VersionUpdateEventArgs {
    pub version: String,
    pub flavor: String,
    pub status: VersionStatus,
}

impl VersionService {
    pub fn new(
        client: ClientWithMiddleware,
        install_service: InstallService,
        task_service: TaskService,
    ) -> Self {
        let update_event = VersionUpdateEvent::new(task_service.clone());
        Self {
            client,
            install_service,
            task_service,
            update_event,
        }
    }

    pub fn update_event(&self) -> &VersionUpdateEvent {
        &self.update_event
    }

    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = crate::godot_website::get_versions(&self.client).await?;
        let installs = &self.list_installs().await?;
        let tasks = &self.list_tasks();
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| {
                let key = (version.name.clone(), version.flavor.clone());
                let status = if installs.contains(&key) {
                    VersionStatus::Installed
                } else if let Some(task) = tasks.get(&key) {
                    match &task.status {
                        TaskStatus::Completed => VersionStatus::Installed,
                        TaskStatus::Failed(e) => VersionStatus::Failed(e.clone()),
                        _ => VersionStatus::Installing,
                    }
                } else {
                    VersionStatus::Available
                };

                Version {
                    name: version.name,
                    flavor: version.flavor,
                    release_notes: format!(
                        "https://godotengine.org/{}",
                        version.release_notes.trim_start_matches("/")
                    ),
                    status,
                }
            })
            .collect())
    }

    async fn list_installs(&self) -> Result<HashSet<(String, String)>, Error> {
        let installs = self.install_service.list().await?;
        Ok(installs
            .into_iter()
            .map(|install| (install.metadata.version, install.metadata.flavor))
            .collect())
    }

    fn list_tasks(&self) -> HashMap<(String, String), Task> {
        let tasks = self.task_service.list();
        tasks
            .into_iter()
            .map(|task| ((task.version.clone(), task.flavor.clone()), task))
            .collect()
    }
}

impl VersionUpdateEvent {
    pub fn new(task_service: TaskService) -> Self {
        Self { task_service }
    }

    pub fn subscribe<F>(&self, f: F)
    where
        F: Fn(VersionUpdateEventArgs) + Send + Sync + 'static,
    {
        self.task_service.update_event().subscribe(move |task| {
            let args = VersionUpdateEventArgs {
                version: task.version,
                flavor: task.flavor,
                status: match task.status {
                    TaskStatus::Completed => VersionStatus::Installed,
                    TaskStatus::Failed(e) => VersionStatus::Failed(e),
                    _ => VersionStatus::Installing,
                },
            };

            f(args);
        });
    }
}
