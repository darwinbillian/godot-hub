use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use reqwest_middleware::ClientWithMiddleware;

use crate::{
    error::Error,
    event::EventHandler,
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

#[derive(Clone)]
pub enum VersionStatus {
    Available,
    Installing,
    Installed,
    Failed(Arc<Error>),
}

pub struct VersionUpdateEvent {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<VersionUpdateEventArgs> + Send + Sync>>>>,
}

pub struct VersionUpdateEventAdapter {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<VersionUpdateEventArgs> + Send + Sync>>>>,
}

#[derive(Clone)]
pub struct VersionUpdateEventArgs {
    pub name: String,
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
        let handlers = Arc::new(Mutex::new(Vec::new()));

        install_service
            .update_event()
            .subscribe(VersionUpdateEventAdapter {
                handlers: handlers.clone(),
            });

        Self { handlers }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<VersionUpdateEventArgs> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler));
    }
}

impl EventHandler<InstallUpdateEventArgs> for VersionUpdateEventAdapter {
    fn invoke(&self, args: InstallUpdateEventArgs) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(VersionUpdateEventArgs::from(args));
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
            name: value.version,
            flavor: value.flavor,
            status: value.status.into(),
        }
    }
}
