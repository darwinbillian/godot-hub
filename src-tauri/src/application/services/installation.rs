use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::application::utils::event::Event;

#[derive(Clone)]
pub struct InstallationService {
    inner: Arc<InstallationServiceInner>,
}

pub struct Installation {
    pub dir: PathBuf,
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub executable: String,
}

pub struct InstallationTransaction {
    dir: PathBuf,
    id: String,
    version: String,
    flavor: String,
}

pub struct InstallationHandle {
    remove_event: Event<InstallationRemoveEventArgs>,
    dir: PathBuf,
    id: String,
    executable: String,
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

struct InstallationServiceInner {
    remove_event: Event<InstallationRemoveEventArgs>,
    dir: PathBuf,
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

    pub fn create(&self, id: &str, version: &str, flavor: &str) -> InstallationTransaction {
        InstallationTransaction {
            dir: self.inner.dir.join(id),
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
        }
    }

    pub async fn list(&self) -> Result<Vec<Installation>> {
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
                dir,
                id,
                version: metadata.version,
                flavor: metadata.flavor,
                executable: metadata.executable,
            };

            installations.push(installation);
        }

        Ok(installations)
    }

    pub async fn get(&self, id: &str) -> Result<InstallationHandle> {
        let dir = self.inner.dir.join(id);
        let metadata = InstallationMetadata::load(&dir).await?;
        let installation = InstallationHandle::new(&dir, id, &metadata.executable);

        installation
            .remove_event()
            .subscribe(self.inner.remove_event.clone());

        Ok(installation)
    }
}

impl InstallationHandle {
    pub fn new(dir: &Path, id: &str, executable: &str) -> Self {
        Self {
            remove_event: Event::new(),
            dir: dir.to_owned(),
            id: id.to_owned(),
            executable: executable.to_owned(),
        }
    }

    pub fn remove_event(&self) -> &Event<InstallationRemoveEventArgs> {
        &self.remove_event
    }

    pub fn launch(&self) -> Result<()> {
        let executable = self.dir.join(&self.executable);
        Command::new(executable).spawn()?;
        Ok(())
    }

    pub async fn uninstall(&self) -> Result<()> {
        tokio::fs::remove_dir_all(&self.dir).await?;

        let args = InstallationRemoveEventArgs::new(&self.id);
        self.remove_event.invoke(Arc::new(args));

        Ok(())
    }

    pub fn reveal(&self) -> Result<()> {
        let executable = self.dir.join(&self.executable);
        tauri_plugin_opener::reveal_item_in_dir(executable)?;
        Ok(())
    }
}

impl InstallationTransaction {
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub async fn commit(self, executable: &str) -> Result<Installation> {
        let installation = Installation {
            dir: self.dir,
            id: self.id,
            version: self.version,
            flavor: self.flavor,
            executable: executable.to_owned(),
        };

        let metadata = InstallationMetadata::from(&installation);
        metadata.save(&installation.dir).await?;

        Ok(installation)
    }
}

impl InstallationMetadata {
    async fn save(&self, dir: &Path) -> Result<()> {
        let bytes = serde_json::to_vec(self)?;
        let path = dir.join("metadata.hub.json");
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    async fn load(dir: &Path) -> Result<InstallationMetadata> {
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

impl<I> From<I> for InstallationMetadata
where
    I: Borrow<Installation>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            version: value.version.clone(),
            flavor: value.flavor.clone(),
            executable: value.executable.clone(),
        }
    }
}
