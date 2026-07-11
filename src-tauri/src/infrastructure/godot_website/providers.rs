use tokio_stream::StreamExt;

use super::client::GodotWebsiteClient;
use crate::application::{
    error::Error,
    services::{
        download::{DownloadProvider, DownloadRequest, DownloadResponse},
        version::{RemoteVersion, VersionProvider},
    },
};

pub struct GodotWebsiteVersionProvider {
    client: GodotWebsiteClient,
}

pub struct GodotWebsiteDownloadProvider {
    client: GodotWebsiteClient,
}

impl GodotWebsiteVersionProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

impl GodotWebsiteDownloadProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl VersionProvider for GodotWebsiteVersionProvider {
    async fn list_versions(&self) -> Result<Vec<RemoteVersion>, Error> {
        let versions = self.client.list_versions().await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| RemoteVersion {
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

#[async_trait::async_trait]
impl DownloadProvider for GodotWebsiteDownloadProvider {
    async fn download(&self, download: DownloadRequest) -> Result<DownloadResponse, Error> {
        let response = self.client.download(download).await?;
        let size = response.content_length();
        let stream = response.bytes_stream();

        let response = DownloadResponse {
            stream: Box::pin(stream.map(|chunk| chunk.map_err(Error::from))),
            size,
        };

        Ok(response)
    }
}
