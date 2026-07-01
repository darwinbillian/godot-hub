use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    error::Error,
    event::EventHandler,
    services::{
        download::DownloadService,
        task::{Task, TaskService, TaskStatus, TaskUpdateEventArgs},
    },
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
}

pub struct InstallServiceInner {
    download_service: DownloadService,
    task_service: TaskService,
    update_event: InstallUpdateEvent,
    remove_event: InstallRemoveEvent,
    dir: PathBuf,
}

pub struct Install {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: InstallStatus,
}

#[derive(Clone)]
pub enum InstallStatus {
    Installing,
    Installed(Arc<Installation>),
    Failed(Arc<Error>),
}

pub struct Installation {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub dir: PathBuf,
}

pub struct InstallationHandle {
    remove_event: InstallRemoveEvent,
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

pub struct InstallUpdateEvent {
    task_service: TaskService,
}

pub struct InstallUpdateEventArgs {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: InstallStatus,
}

pub struct InstallRemoveEvent {
    handlers: Mutex<Vec<Arc<dyn EventHandler<InstallRemoveEventArgs> + Send + Sync>>>,
}

#[derive(Clone)]
pub struct InstallRemoveEventArgs {
    pub id: String,
}

impl InstallService {
    pub fn new(download_service: DownloadService, task_service: TaskService, dir: PathBuf) -> Self {
        let update_event = InstallUpdateEvent::new(task_service.clone());
        Self {
            inner: Arc::new(InstallServiceInner {
                download_service,
                task_service,
                update_event,
                remove_event: InstallRemoveEvent::new(),
                dir,
            }),
        }
    }

    pub fn update_event(&self) -> &InstallUpdateEvent {
        &self.inner.update_event
    }

    pub fn remove_event(&self) -> &InstallRemoveEvent {
        &self.inner.remove_event
    }

    pub async fn install(&self, version: &str, flavor: &str) -> Result<(), Error> {
        let id = format!("{}-{}", version, flavor);
        let task = Task::new(&id, version, flavor);

        self.inner
            .task_service
            .start(task, async || -> Result<Installation, Error> {
                let download_path = self.download(version, flavor).await?;

                let dir = self.inner.dir.join(&id);
                crate::utils::zip::extract(download_path, &dir).await?;

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
            let install = Install {
                id: task.id,
                flavor: task.flavor,
                version: task.version,
                status: task.status.into(),
            };
            installs.insert(install.id.clone(), install);
        }

        for installation in installations {
            let install = Install {
                id: installation.id.clone(),
                version: installation.version.clone(),
                flavor: installation.flavor.clone(),
                status: InstallStatus::Installed(Arc::new(installation)),
            };
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

        let inner = self.inner.clone();
        install.remove_event().subscribe(move |args| {
            inner.remove_event.invoke(args);
        });

        Ok(install)
    }

    async fn download(&self, version: &str, flavor: &str) -> Result<PathBuf, Error> {
        let url = format!("https://downloads.godotengine.org/?version={}&flavor={}&slug=win64.exe.zip&platform=windows.64", version, flavor);
        let name = format!("Godot_v{}-{}_win64.exe.zip", version, flavor);
        let path = self.inner.download_service.download(&url, &name).await?;
        Ok(path)
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
            remove_event: InstallRemoveEvent::new(),
            id: id.to_owned(),
            dir: dir.to_owned(),
            executable: dir.join(executable),
        }
    }

    pub fn remove_event(&self) -> &InstallRemoveEvent {
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
        self.remove_event.invoke(args);

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

impl InstallUpdateEvent {
    pub fn new(task_service: TaskService) -> Self {
        Self { task_service }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<InstallUpdateEventArgs> + Send + Sync + 'static,
    {
        self.task_service
            .update_event()
            .subscribe(move |args: TaskUpdateEventArgs| -> () {
                handler.invoke(args.into());
            });
    }
}

impl InstallRemoveEvent {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<InstallRemoveEventArgs> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler));
    }

    pub fn invoke(&self, args: InstallRemoveEventArgs) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(args);
    }
}

impl From<TaskStatus> for InstallStatus {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Completed(installation) => InstallStatus::Installed(installation),
            TaskStatus::Failed(e) => InstallStatus::Failed(e),
            _ => InstallStatus::Installing,
        }
    }
}

impl From<TaskUpdateEventArgs> for InstallUpdateEventArgs {
    fn from(value: TaskUpdateEventArgs) -> Self {
        Self {
            id: value.id,
            version: value.version,
            flavor: value.flavor,
            status: value.status.into(),
        }
    }
}
