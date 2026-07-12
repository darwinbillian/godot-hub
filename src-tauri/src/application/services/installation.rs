use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::application::{error::Error, event::Event};

#[derive(Clone)]
pub struct InstallationService {
    inner: Arc<InstallationServiceInner>,
}

pub struct InstallationServiceInner {
    remove_event: Event<InstallationRemoveEventArgs>,
    dir: PathBuf,
}

pub struct Installation {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub dir: PathBuf,
}

pub struct InstallationHandle {
    remove_event: Event<InstallationRemoveEventArgs>,
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

pub struct InstallationRemoveEventArgs {
    pub id: String,
}

impl InstallationService {
    pub fn new(dir: &Path) -> Self {
        Self {
            inner: Arc::new(InstallationServiceInner {
                remove_event: Event::new(),
                dir: dir.to_owned(),
            }),
        }
    }

    pub fn remove_event(&self) -> &Event<InstallationRemoveEventArgs> {
        &self.inner.remove_event
    }

    pub fn create(&self, id: &str, version: &str, flavor: &str) -> Installation {
        Installation {
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            dir: self.inner.dir.join(id),
        }
    }

    pub async fn list(&self) -> Result<Vec<Installation>, Error> {
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

    pub async fn get(&self, id: &str) -> Result<InstallationHandle, Error> {
        let dir = self.inner.dir.join(id);
        let metadata = InstallationMetadata::load(&dir).await?;
        let installation = InstallationHandle::new(id, &dir, &metadata.executable);

        installation
            .remove_event()
            .subscribe(self.inner.remove_event.clone());

        Ok(installation)
    }
}

impl InstallationHandle {
    pub fn new(id: &str, dir: &Path, executable: &str) -> Self {
        Self {
            remove_event: Event::new(),
            id: id.to_owned(),
            dir: dir.to_owned(),
            executable: dir.join(executable),
        }
    }

    pub fn remove_event(&self) -> &Event<InstallationRemoveEventArgs> {
        &self.remove_event
    }

    pub async fn launch(&self) -> Result<(), Error> {
        Command::new(&self.executable).spawn()?;
        Ok(())
    }

    pub async fn uninstall(&self) -> Result<(), Error> {
        tokio::fs::remove_dir_all(&self.dir).await?;

        let args = InstallationRemoveEventArgs::new(&self.id);
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

impl InstallationRemoveEventArgs {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}
