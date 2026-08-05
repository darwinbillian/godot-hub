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
    domain::models::version::{Flavor, Variant, Version, VersionFlavor, VersionFlavorVariant},
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
                let (version, flavor) = match (version.parse::<Version>(), flavor.parse::<Flavor>())
                {
                    (Ok(version), Ok(flavor)) => (version, flavor),
                    _ => return None,
                };

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
        variant: Variant,
        platform: &str,
    ) -> Result<String, DownloadConfigsError> {
        let major = version.major.to_string();
        let current = VersionFlavor::new(version, flavor);

        let group = self
            .download_configs
            .overrides
            .iter()
            .rfind(|o| {
                if o.version != major {
                    return false;
                }

                let (lower, upper) = match o.range.as_slice() {
                    [lower, upper] => (lower, upper),
                    _ => return false,
                };

                let (lower, upper) = match (
                    lower.parse::<VersionFlavor>(),
                    upper.parse::<VersionFlavor>(),
                ) {
                    (Ok(lower), Ok(upper)) => (lower, upper),
                    _ => return false,
                };

                current >= lower && current <= upper
            })
            .map(|o| &o.config)
            .or_else(|| self.download_configs.defaults.get(&major))
            .ok_or_else(|| {
                DownloadConfigsError::ReleaseNotAvailable(VersionFlavorVariant::new(
                    version, flavor, variant,
                ))
            })?;

        let download_config = match variant {
            Variant::Standard => &group.standard,
            Variant::Mono => group.mono.as_ref().ok_or_else(|| {
                DownloadConfigsError::ReleaseNotAvailable(VersionFlavorVariant::new(
                    version, flavor, variant,
                ))
            })?,
        };

        let editor = download_config.editor.as_ref().ok_or_else(|| {
            DownloadConfigsError::ReleaseNotAvailable(VersionFlavorVariant::new(
                version, flavor, variant,
            ))
        })?;

        let slug = editor.get(platform).ok_or_else(|| {
            DownloadConfigsError::ReleaseNotAvailableForPlatform(
                VersionFlavorVariant::new(version, flavor, variant),
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
