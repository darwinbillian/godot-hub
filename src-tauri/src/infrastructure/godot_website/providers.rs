use std::str::FromStr;

use anyhow::{Error, Result};
use tokio_stream::StreamExt;

use super::client::GodotWebsiteClient;
use crate::{
    application::services::{
        download::{DownloadProvider, DownloadRequest, DownloadResponse},
        installer::{DownloadConfigsProvider, InstallerError},
        release::{ReleaseMetadata, ReleaseProvider},
    },
    domain::models::version::Version,
};

pub struct GodotWebsiteReleaseProvider {
    client: GodotWebsiteClient,
}

impl GodotWebsiteReleaseProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl ReleaseProvider for GodotWebsiteReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>> {
        let versions = self.client.list_versions().await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| ReleaseMetadata {
                name: version.name,
                flavor: version.flavor,
                release_notes: format!(
                    "https://godotengine.org/{}",
                    version.release_notes.trim_start_matches("/")
                ),
            })
            .collect())
    }
}

pub struct GodotWebsiteDownloadConfigsProvider {
    client: GodotWebsiteClient,
}

impl GodotWebsiteDownloadConfigsProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl DownloadConfigsProvider for GodotWebsiteDownloadConfigsProvider {
    async fn get_slug(&self, version: &str, _flavor: &str, platform: &str) -> Result<String> {
        let version = Version::from_str(version)?;
        let download_configs = self.client.list_download_configs().await?;
        let config = download_configs
            .defaults
            .get(&version.major.to_string())
            .ok_or_else(|| InstallerError::VersionNotAvailable(version.clone()))?;
        let editor = config
            .editor
            .as_ref()
            .ok_or_else(|| InstallerError::VersionNotAvailable(version.clone()))?;
        let slug = editor
            .get(platform)
            .ok_or_else(|| InstallerError::VersionNotAvailable(version.clone()))?;
        Ok(slug.to_owned())
    }
}

pub struct GodotWebsiteDownloadProvider {
    client: GodotWebsiteClient,
}

impl GodotWebsiteDownloadProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl DownloadProvider for GodotWebsiteDownloadProvider {
    async fn download(&self, request: DownloadRequest) -> Result<DownloadResponse> {
        let response = self
            .client
            .download(
                &request.version,
                &request.flavor,
                &request.slug,
                &request.platform,
            )
            .await?;

        let size = response.content_length();
        let stream = response.bytes_stream();

        let response = DownloadResponse {
            stream: Box::pin(stream.map(|chunk| chunk.map_err(Error::from))),
            size,
        };

        Ok(response)
    }
}
