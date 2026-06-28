use std::collections::HashSet;

use reqwest_middleware::ClientWithMiddleware;

use crate::{error::Error, services::install::InstallService};

pub struct VersionService {
    client: ClientWithMiddleware,
    install_service: InstallService,
}

pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
    pub status: VersionStatus,
}

pub enum VersionStatus {
    Available,
    Installed,
}

impl VersionService {
    pub fn new(client: ClientWithMiddleware, install_service: InstallService) -> Self {
        Self {
            client,
            install_service,
        }
    }

    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = crate::godot_website::get_versions(&self.client).await?;
        let installs = &self.list_installs().await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| {
                let key = (version.name.clone(), version.flavor.clone());
                let status = if installs.contains(&key) {
                    VersionStatus::Installed
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

    async fn list_installs(&self) -> Result<HashSet<(String, String)>, Error> {
        let installs = self.install_service.list().await?;
        Ok(installs
            .into_iter()
            .map(|install| (install.metadata.version, install.metadata.flavor))
            .collect())
    }
}
