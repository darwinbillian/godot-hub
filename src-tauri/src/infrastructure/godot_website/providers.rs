use anyhow::{Error, Result};
use tokio_stream::StreamExt;

use super::client::GodotWebsiteClient;
use crate::application::services::{
    download::{DownloadProvider, DownloadRequest, DownloadResponse},
    release::{ReleaseMetadata, ReleaseProvider},
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
