use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use itertools::Itertools;

use crate::{
    application::{
        interfaces::{
            download_configs::{DownloadConfigsError, DownloadConfigsProvider},
            release::ReleaseProvider,
        },
        services::{
            install::{Install, InstallService},
            platform::PlatformService,
        },
    },
    domain::models::{Flavor, FlavorKindFlags, ReleaseVariant, Variant, VariantFlags, Version},
};

pub struct ReleaseService {
    download_configs_provider: Arc<dyn DownloadConfigsProvider + Send + Sync>,
    release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
    install_service: InstallService,
    platform_service: PlatformService,
}

impl ReleaseService {
    pub fn new(
        download_configs_provider: Arc<dyn DownloadConfigsProvider + Send + Sync>,
        release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
        install_service: InstallService,
        platform_service: PlatformService,
    ) -> Self {
        Self {
            download_configs_provider,
            release_provider,
            install_service,
            platform_service,
        }
    }

    pub async fn list(&self, filter: Option<ReleaseFilter>) -> Result<Vec<Release>> {
        let platform = self.platform_service.get_platform()?;
        let download_configs = self
            .download_configs_provider
            .get_download_configs()
            .await?;

        let variants = Variant::iter()
            .filter(|variant| {
                filter
                    .as_ref()
                    .and_then(|filter| filter.variant)
                    .unwrap_or_default()
                    .contains(*variant)
            })
            .collect::<Vec<Variant>>();

        let installs = self
            .install_service
            .list(None)
            .await?
            .into_iter()
            .map(|install| {
                let id = format!(
                    "{}",
                    ReleaseVariant::new(install.version, install.flavor, install.variant)
                );

                (id, install)
            })
            .collect::<HashMap<String, Install>>();

        let releases = self
            .release_provider
            .list_releases()
            .await?
            .iter()
            .filter(|metadata| {
                filter
                    .as_ref()
                    .and_then(|filter| filter.flavor)
                    .unwrap_or_default()
                    .contains(metadata.flavor)
            })
            .cartesian_product(variants)
            .filter_map(|(metadata, variant)| {
                let id = format!(
                    "{}",
                    ReleaseVariant::new(metadata.version, metadata.flavor, variant)
                );

                let name = format!(
                    "{:#}",
                    ReleaseVariant::new(metadata.version, metadata.flavor, variant)
                );

                let status = match download_configs.get_slug(
                    metadata.version,
                    metadata.flavor,
                    variant,
                    &platform,
                ) {
                    Ok(_) => ReleaseStatus::Available,
                    Err(e) => match e {
                        DownloadConfigsError::ReleaseNotAvailableForPlatform(_, _) => {
                            ReleaseStatus::Unavailable
                        }
                        _ => return None,
                    },
                };

                let install = installs.get(&id).cloned();

                let release = Release {
                    id,
                    name,
                    version: metadata.version,
                    flavor: metadata.flavor,
                    variant,
                    release_notes: metadata.release_notes.clone(),
                    status,
                    install,
                };

                Some(release)
            })
            .collect::<Vec<Release>>();

        Ok(releases)
    }
}

pub struct Release {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub flavor: Flavor,
    pub variant: Variant,
    pub release_notes: String,
    pub status: ReleaseStatus,
    pub install: Option<Install>,
}

pub enum ReleaseStatus {
    Available,
    Unavailable,
}

#[derive(Default)]
pub struct ReleaseFilter {
    pub flavor: Option<FlavorKindFlags>,
    pub variant: Option<VariantFlags>,
}
