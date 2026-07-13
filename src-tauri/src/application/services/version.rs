use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::application::{
    error::Error,
    services::install::{Install, InstallService, InstallStatus, InstallUpdateEventArgs},
    utils::event::Event,
};

#[async_trait::async_trait]
pub trait VersionProvider {
    async fn list_versions(&self) -> Result<Vec<RemoteVersion>, Error>;
}

pub struct VersionService {
    version_provider: Arc<dyn VersionProvider + Send + Sync>,
    install_service: InstallService,
    update_event: Event<VersionUpdateEventArgs>,
}

pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: VersionStatus,
}

pub struct RemoteVersion {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
}

pub enum VersionStatus {
    Available,
    Installing,
    Installed,
    Failed(Arc<Error>),
}

pub struct VersionUpdateEventArgs {
    pub name: String,
    pub flavor: String,
    pub status: VersionStatus,
}

impl VersionService {
    pub fn new(
        version_provider: Arc<dyn VersionProvider + Send + Sync>,
        install_service: InstallService,
    ) -> Self {
        let update_event = Event::new();

        install_service
            .update_event()
            .map(VersionUpdateEventArgs::from)
            .subscribe(update_event.clone());

        Self {
            version_provider,
            install_service,
            update_event,
        }
    }

    pub fn update_event(&self) -> &Event<VersionUpdateEventArgs> {
        &self.update_event
    }

    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = self.version_provider.list_versions().await?;
        let installs = self.list_installs().await?;
        Ok(versions
            .into_iter()
            .map(|version| {
                let key = (version.name.clone(), version.flavor.clone());
                let status = if let Some(install) = installs.get(&key) {
                    VersionStatus::from(&install.status)
                } else {
                    VersionStatus::Available
                };

                Version {
                    name: version.name,
                    flavor: version.flavor,
                    release_notes: version.release_notes,
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

impl<I> From<I> for VersionStatus
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

impl<I> From<I> for VersionUpdateEventArgs
where
    I: Borrow<InstallUpdateEventArgs>,
{
    fn from(value: I) -> Self {
        let value = value.borrow();
        Self {
            name: value.version.clone(),
            flavor: value.flavor.clone(),
            status: VersionStatus::from(&value.status),
        }
    }
}
