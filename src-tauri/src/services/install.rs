use std::path::{Path, PathBuf};

use http_cache_reqwest::CacheMode;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::error::Error;

pub struct InstallService {
    pub client: ClientWithMiddleware,
    pub downloads_dir: PathBuf,
    pub installs_dir: PathBuf,
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
        let dir = self.installs_dir.join(id);
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

    async fn download(&self, version: &str, flavor: &str) -> Result<PathBuf, Error> {
        let request = self
            .client
            .get(format!("https://downloads.godotengine.org/?version={}&flavor={}&slug=win64.exe.zip&platform=windows.64", version, flavor))
            .with_extension(CacheMode::NoStore);
        let response = request.send().await?.error_for_status()?;
        let mut stream = response.bytes_stream();

        tokio::fs::create_dir_all(&self.downloads_dir).await?;

        let path = self
            .downloads_dir
            .join(format!("Godot_v{}-{}_win64.exe.zip", version, flavor));
        let mut file = File::create(&path).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

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
}
