use std::{collections::HashMap, sync::Arc};

use crate::application::{
    error::Error,
    services::install::{Install, InstallService},
};

#[async_trait::async_trait]
pub trait ReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>, Error>;
}

pub struct ReleaseService {
    release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
    install_service: InstallService,
}

pub struct Release {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: ReleaseStatus,
    pub install: Option<Install>,
}

pub struct ReleaseMetadata {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
}

pub enum ReleaseStatus {
    Available,
}

impl ReleaseService {
    pub fn new(
        release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
        install_service: InstallService,
    ) -> Self {
        Self {
            release_provider,
            install_service,
        }
    }

    pub async fn list(&self) -> Result<Vec<Release>, Error> {
        let releases = self.release_provider.list_releases().await?;
        let installs = self.list_installs().await?;
        Ok(releases
            .into_iter()
            .map(|release| {
                let key = (release.name.clone(), release.flavor.clone());
                Release {
                    name: release.name,
                    flavor: release.flavor,
                    release_notes: release.release_notes,
                    status: ReleaseStatus::Available,
                    install: installs.get(&key).cloned(),
                }
            })
            .collect())
    }

    async fn list_installs(&self) -> Result<HashMap<(String, String), Install>, Error> {
        let installs = self.install_service.list().await?;
        Ok(installs
            .into_iter()
            .map(|install| ((install.version.clone(), install.flavor.clone()), install))
            .collect())
    }
}
