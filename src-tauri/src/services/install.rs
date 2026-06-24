use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{error::Error, services::download::DownloadService};

pub struct InstallService {
    pub download_service: DownloadService,
    pub dir: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Install {
    pub id: String,
    pub dir: String,
    pub metadata: InstallMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallMetadata {
    pub version: String,
    pub flavor: String,
    pub executable: String,
}

impl InstallService {
    pub async fn install(&self, version: &str, flavor: &str) -> Result<(), Error> {
        let download_path = self.download(version, flavor).await?;

        let id = format!("{}-{}", version, flavor);
        let dir = self.dir.join(id);
        crate::utils::zip::extract(download_path, &dir).await?;

        let executable = format!("Godot_v{}-{}_win64.exe", version, flavor);
        let metadata = InstallMetadata {
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            executable,
        };
        metadata.save(&dir).await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Install>, Error> {
        let mut installs = Vec::<Install>::new();

        let mut entries = match tokio::fs::read_dir(&self.dir).await {
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

            let install = Install {
                id,
                dir: dir.to_string_lossy().into_owned(),
                metadata,
            };
            installs.push(install);
        }

        Ok(installs)
    }

    pub async fn launch(&self, id: &str) -> Result<(), Error> {
        let dir = self.dir.join(id);
        let metadata = InstallMetadata::load(&dir).await?;
        let executable = dir.join(&metadata.executable);
        Command::new(executable).spawn()?;
        Ok(())
    }

    pub async fn uninstall(&self, id: &str) -> Result<(), Error> {
        let dir = self.dir.join(id);
        tokio::fs::remove_dir_all(dir).await?;
        Ok(())
    }

    async fn download(&self, version: &str, flavor: &str) -> Result<PathBuf, Error> {
        let url = format!("https://downloads.godotengine.org/?version={}&flavor={}&slug=win64.exe.zip&platform=windows.64", version, flavor);
        let name = format!("Godot_v{}-{}_win64.exe.zip", version, flavor);
        let path = self.download_service.download(&url, &name).await?;
        Ok(path)
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
