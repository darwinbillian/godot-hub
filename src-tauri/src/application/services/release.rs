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
    domain::models::version::{Flavor, Version},
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
                [false, true].into_iter().filter_map(|mono| {
                    let id = format!(
                        "{}-{}{}",
                        metadata.version,
                        metadata.flavor,
                        if mono { "-mono" } else { "" }
                    );

                    let name = format!(
                        "Godot {:.2}{}",
                        metadata.version,
                        if mono { " Mono" } else { "" }
                    );

                    let status = match download_configs.get_slug(
                        metadata.version,
                        metadata.flavor,
                        mono,
                        &platform,
                    ) {
                        Ok(_) => ReleaseStatus::Available,
                        Err(e) => match e {
                            DownloadConfigsError::ReleaseNotAvailableForPlatform(_, _, _) => {
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
                        mono,
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
                    "{}-{}{}",
                    install.version,
                    install.flavor,
                    if install.mono { "-mono" } else { "" }
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
    pub mono: bool,
    pub release_notes: String,
    pub status: ReleaseStatus,
    pub install: Option<Install>,
}

pub enum ReleaseStatus {
    Available,
    Unavailable,
}
