use std::{collections::HashMap, sync::Arc};

use reqwest_middleware::ClientWithMiddleware;

use crate::{
    error::Error,
    services::install::{Install, InstallService, InstallStatus, InstallUpdateEventArgs},
};

pub struct VersionService {
    client: ClientWithMiddleware,
    install_service: InstallService,
    update_event: VersionUpdateEvent,
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

pub struct VersionUpdateEvent {
    install_service: InstallService,
}

pub struct VersionUpdateEventArgs {
    pub version: String,
    pub flavor: String,
    pub status: VersionStatus,
}

impl VersionService {
    pub fn new(client: ClientWithMiddleware, install_service: InstallService) -> Self {
        let update_event = VersionUpdateEvent::new(install_service.clone());
        Self {
            client,
            install_service,
            update_event,
        }
    }

    pub fn update_event(&self) -> &VersionUpdateEvent {
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
                    install.status.clone().into()
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

impl VersionUpdateEvent {
    pub fn new(install_service: InstallService) -> Self {
        Self { install_service }
    }

    pub fn subscribe<F>(&self, f: F)
    where
        F: Fn(VersionUpdateEventArgs) + Send + Sync + 'static,
    {
        self.install_service.update_event().subscribe(move |args| {
            f(args.into());
        });
    }
}

impl From<InstallStatus> for VersionStatus {
    fn from(value: InstallStatus) -> Self {
        match value {
            InstallStatus::Installing => VersionStatus::Installing,
            InstallStatus::Installed(_) => VersionStatus::Installed,
            InstallStatus::Failed(e) => VersionStatus::Failed(e),
        }
    }
}

impl From<InstallUpdateEventArgs> for VersionUpdateEventArgs {
    fn from(value: InstallUpdateEventArgs) -> Self {
        Self {
            version: value.version,
            flavor: value.flavor,
            status: value.status.into(),
        }
    }
}
