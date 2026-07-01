use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use reqwest_middleware::ClientWithMiddleware;

use crate::{
    error::Error,
    event::EventAdapter,
    services::install::{Install, InstallService, InstallStatus, InstallUpdateEventArgs},
};

pub struct VersionService {
    client: ClientWithMiddleware,
    install_service: InstallService,
    update_event: EventAdapter<VersionUpdateEventArgs>,
}

pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: VersionStatus,
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
    pub fn new(client: ClientWithMiddleware, install_service: InstallService) -> Self {
        let update_event = EventAdapter::new(install_service.update_event());
        Self {
            client,
            install_service,
            update_event,
        }
    }

    pub fn update_event(&self) -> &EventAdapter<VersionUpdateEventArgs> {
        &self.update_event
    }

    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = crate::godot_website::get_versions(&self.client).await?;
        let installs = self.list_installs().await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
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
                    release_notes: format!(
                        "https://godotengine.org/{}",
                        version.release_notes.trim_start_matches("/")
                    ),
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
        match value.borrow() {
            InstallStatus::Installing => Self::Installing,
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
