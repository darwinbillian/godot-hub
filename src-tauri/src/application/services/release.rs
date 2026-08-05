use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

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
    domain::models::version::{Flavor, Variant, Version, VersionFlavorVariant},
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

    pub async fn list(&self) -> Result<Vec<Release>> {
        let platform = self.platform_service.get_platform()?;
        let download_configs = self
            .download_configs_provider
            .get_download_configs()
            .await?;

        let installs = self.list_installs().await?;
        let releases = self.release_provider.list_releases().await?;
        Ok(releases
            .iter()
            .flat_map(|metadata| {
                Variant::iter().filter_map(|variant| {
                    let id = format!(
                        "{}",
                        VersionFlavorVariant::new(metadata.version, metadata.flavor, variant)
                    );

                    let name = format!(
                        "{:#}",
                        VersionFlavorVariant::new(metadata.version, metadata.flavor, variant)
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
            })
            .collect())
    }

    async fn list_installs(&self) -> Result<HashMap<String, Install>> {
        let installs = self.install_service.list().await?;
        Ok(installs
            .into_iter()
            .map(|install| {
                let id = format!(
                    "{}",
                    VersionFlavorVariant::new(install.version, install.flavor, install.variant)
                );

                (id, install)
            })
            .collect())
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
