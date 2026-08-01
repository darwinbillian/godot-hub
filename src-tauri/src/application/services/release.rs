use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

use crate::application::{
    interfaces::{download_configs::DownloadConfigsProvider, release::ReleaseProvider},
    services::{
        install::{Install, InstallService},
        platform::PlatformService,
    },
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
        let releases = self.release_provider.list_releases().await?;
        let installs = self.list_installs().await?;
        Ok(releases
            .into_iter()
            .map(|release| {
                let key = (release.version.clone(), release.flavor.clone());
                let name = format!("Godot {}", release.version);
                let status =
                    match download_configs.get_slug(&release.version, &release.flavor, &platform) {
                        Ok(_) => ReleaseStatus::Available,
                        Err(_) => ReleaseStatus::Unavailable,
                    };

                Release {
                    name,
                    version: release.version,
                    flavor: release.flavor,
                    release_notes: release.release_notes,
                    status,
                    install: installs.get(&key).cloned(),
                }
            })
            .collect())
    }

    async fn list_installs(&self) -> Result<HashMap<(String, String), Install>> {
        let installs = self.install_service.list().await?;
        Ok(installs
            .into_iter()
            .map(|install| ((install.version.clone(), install.flavor.clone()), install))
            .collect())
    }
}

pub struct Release {
    pub name: String,
    pub version: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: ReleaseStatus,
    pub install: Option<Install>,
}

pub enum ReleaseStatus {
    Available,
    Unavailable,
}
