use anyhow::{Error, Result};
use tokio_stream::StreamExt;

use super::client::GodotWebsiteClient;
use crate::{
    application::interfaces::{
        download::{DownloadProvider, DownloadRequest, DownloadResponse},
        download_configs::{DownloadConfigs, DownloadConfigsProvider},
        release::{ReleaseMetadata, ReleaseProvider},
    },
    domain::models::version::{Flavor, Version},
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
        let releases = self.client.list_versions().await?;
        Ok(releases
            .into_iter()
            .flat_map(|release| {
                std::iter::once((release.name.clone(), release.flavor, release.release_notes))
                    .chain(
                        release
                            .releases
                            .into_iter()
                            .flatten()
                            .map(move |prerelease| {
                                (
                                    release.name.clone(),
                                    prerelease.name,
                                    prerelease.release_notes,
                                )
                            }),
                    )
            })
            .filter_map(|(version, flavor, release_notes)| {
                let version = version.parse::<Version>().ok()?;
                let flavor = flavor.parse::<Flavor>().ok()?;

                let release = ReleaseMetadata {
                    version,
                    flavor,
                    release_notes: format!(
                        "https://godotengine.org/{}",
                        release_notes.trim_start_matches('/')
                    ),
                };

                Some(release)
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
    async fn get_download_configs(&self) -> Result<DownloadConfigs> {
        let download_configs = self.client.get_download_configs().await?;
        Ok(download_configs.try_into()?)
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
                &request.version.to_string(),
                &request.flavor.to_string(),
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
