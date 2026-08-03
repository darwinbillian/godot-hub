use std::sync::Arc;

use anyhow::{Error, Result};
use tokio_stream::StreamExt;

use super::client::GodotWebsiteClient;
use crate::{
    application::interfaces::{
        download::{DownloadProvider, DownloadRequest, DownloadResponse},
        download_configs::{DownloadConfigs, DownloadConfigsError, DownloadConfigsProvider},
        release::{ReleaseMetadata, ReleaseProvider},
    },
    domain::models::version::{Flavor, FlavorKind, Version},
    infrastructure::godot_website::dtos::DownloadConfigsDto,
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
            .filter_map(|release| {
                let (version, flavor) = match (
                    release.name.parse::<Version>(),
                    release.flavor.parse::<Flavor>(),
                ) {
                    (Ok(version), Ok(flavor)) => (version, flavor),
                    _ => return None,
                };

                if flavor.kind != FlavorKind::Stable {
                    return None;
                }

                let metadata = ReleaseMetadata {
                    version,
                    flavor,
                    release_notes: format!(
                        "https://godotengine.org/{}",
                        release.release_notes.trim_start_matches("/")
                    ),
                };

                Some(metadata)
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
    async fn get_download_configs(&self) -> Result<Arc<dyn DownloadConfigs + Send + Sync>> {
        let download_configs = self.client.get_download_configs().await?;
        Ok(Arc::new(GodotWebsiteDownloadConfigs::new(download_configs)))
    }
}

pub struct GodotWebsiteDownloadConfigs {
    download_configs: DownloadConfigsDto,
}

impl GodotWebsiteDownloadConfigs {
    pub fn new(download_configs: DownloadConfigsDto) -> Self {
        Self { download_configs }
    }
}

impl DownloadConfigs for GodotWebsiteDownloadConfigs {
    fn get_slug(
        &self,
        version: Version,
        flavor: Flavor,
        mono: bool,
        platform: &str,
    ) -> Result<String, DownloadConfigsError> {
        let editor = self
            .download_configs
            .defaults
            .get(&version.major.to_string())
            .and_then(|config| {
                if mono {
                    config
                        .mono
                        .as_ref()
                        .and_then(|config| config.editor.as_ref())
                } else {
                    config.editor.as_ref()
                }
            })
            .ok_or(DownloadConfigsError::ReleaseNotAvailable(version, flavor))?;

        let slug = editor.get(platform).ok_or_else(|| {
            DownloadConfigsError::ReleaseNotAvailableForPlatform(
                version,
                flavor,
                platform.to_owned(),
            )
        })?;

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
