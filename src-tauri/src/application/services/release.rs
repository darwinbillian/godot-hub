use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::application::{
    error::Error,
    services::install::{Install, InstallService, InstallStatus, InstallUpdateEventArgs},
    utils::event::Event,
};

#[async_trait::async_trait]
pub trait ReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>, Error>;
}

pub struct ReleaseService {
    release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
    install_service: InstallService,
    update_event: Event<ReleaseUpdateEventArgs>,
}

pub struct Release {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: ReleaseStatus,
}

pub struct ReleaseMetadata {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
}

pub enum ReleaseStatus {
    Available,
    Installing,
    Installed,
    Failed(Arc<Error>),
}

pub struct ReleaseUpdateEventArgs {
    pub name: String,
    pub flavor: String,
    pub status: ReleaseStatus,
}

impl ReleaseService {
    pub fn new(
        release_provider: Arc<dyn ReleaseProvider + Send + Sync>,
        install_service: InstallService,
    ) -> Self {
        let update_event = Event::new();

        install_service
            .update_event()
            .map(ReleaseUpdateEventArgs::from)
            .subscribe(update_event.clone());

        Self {
            release_provider,
            install_service,
            update_event,
        }
    }

    pub fn update_event(&self) -> &Event<ReleaseUpdateEventArgs> {
        &self.update_event
    }

    pub async fn list(&self) -> Result<Vec<Release>, Error> {
        let releases = self.release_provider.list_releases().await?;
        let installs = self.list_installs().await?;
        Ok(releases
            .into_iter()
            .map(|release| {
                let key = (release.name.clone(), release.flavor.clone());
                let status = if let Some(install) = installs.get(&key) {
                    ReleaseStatus::from(&install.status)
                } else {
                    ReleaseStatus::Available
                };

                Release {
                    name: release.name,
                    flavor: release.flavor,
                    release_notes: release.release_notes,
                    status,
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

impl<I> From<I> for ReleaseStatus
where
    I: Borrow<InstallStatus>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        match value {
            InstallStatus::Installing(_) => Self::Installing,
            InstallStatus::Installed(_) => Self::Installed,
            InstallStatus::Failed(e) => Self::Failed(e.clone()),
        }
    }
}

impl<I> From<I> for ReleaseUpdateEventArgs
where
    I: Borrow<InstallUpdateEventArgs>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            name: value.version.clone(),
            flavor: value.flavor.clone(),
            status: ReleaseStatus::from(&value.status),
        }
    }
}
