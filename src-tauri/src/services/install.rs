use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    error::Error,
    services::{
        download::DownloadService,
        task::{Task, TaskService},
    },
};

#[derive(Clone)]
pub struct InstallService {
    inner: Arc<InstallServiceInner>,
}

pub struct InstallServiceInner {
    download_service: DownloadService,
    task_service: TaskService,
    dir: PathBuf,
}

pub struct Install {
    pub id: String,
    pub dir: PathBuf,
    pub metadata: InstallMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallMetadata {
    pub version: String,
    pub flavor: String,
    pub executable: String,
}

impl InstallService {
    pub fn new(download_service: DownloadService, task_service: TaskService, dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(InstallServiceInner {
                download_service,
                task_service,
                dir,
            }),
        }
    }

    pub async fn install(&self, version: &str, flavor: &str) -> Result<(), Error> {
        let id = format!("{}-{}", version, flavor);
        let task = Task::new(&id, version, flavor);

        self.inner
            .task_service
            .spawn(task, async || -> Result<(), Error> {
                let download_path = self.download(version, flavor).await?;

                let dir = self.inner.dir.join(id);
                crate::utils::zip::extract(download_path, &dir).await?;

                let executable = format!("Godot_v{}-{}_win64.exe", version, flavor);
                let metadata = InstallMetadata {
                    version: version.to_owned(),
                    flavor: flavor.to_owned(),
                    executable,
                };
                metadata.save(&dir).await?;

                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Install>, Error> {
        let mut installs = Vec::<Install>::new();

        let mut entries = match tokio::fs::read_dir(&self.inner.dir).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(installs),
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
            let metadata = match InstallMetadata::load(&dir).await {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            let install = Install { id, dir, metadata };
            installs.push(install);
        }

        Ok(installs)
    }

    pub async fn get(&self, id: &str) -> Result<Install, Error> {
        let dir = self.inner.dir.join(id);
        let metadata = InstallMetadata::load(&dir).await?;
        let install = Install {
            id: id.to_owned(),
            dir,
            metadata,
        };
        Ok(install)
    }

    async fn download(&self, version: &str, flavor: &str) -> Result<PathBuf, Error> {
        let url = format!("https://downloads.godotengine.org/?version={}&flavor={}&slug=win64.exe.zip&platform=windows.64", version, flavor);
        let name = format!("Godot_v{}-{}_win64.exe.zip", version, flavor);
        let path = self.inner.download_service.download(&url, &name).await?;
        Ok(path)
    }
}

impl Install {
    pub async fn launch(&self) -> Result<(), Error> {
        let executable = self.dir.join(&self.metadata.executable);
        Command::new(executable).spawn()?;
        Ok(())
    }

    pub async fn uninstall(&self) -> Result<(), Error> {
        tokio::fs::remove_dir_all(&self.dir).await?;
        Ok(())
    }

    pub async fn reveal(&self) -> Result<(), Error> {
        let executable = self.dir.join(&self.metadata.executable);
        tauri_plugin_opener::reveal_item_in_dir(executable)?;
        Ok(())
    }
}

impl InstallMetadata {
    pub async fn save(&self, dir: &Path) -> Result<(), Error> {
        let bytes = serde_json::to_vec(self)?;
        let path = dir.join("metadata.hub.json");
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    pub async fn load(dir: &Path) -> Result<InstallMetadata, Error> {
        let path = dir.join("metadata.hub.json");
        let bytes = tokio::fs::read(path).await?;
        let metadata = serde_json::from_slice::<InstallMetadata>(&bytes)?;
        Ok(metadata)
    }
}
