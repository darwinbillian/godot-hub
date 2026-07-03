use crate::application::services::{install::InstallService, version::VersionService};

pub struct AppState {
    pub install_service: InstallService,
    pub version_service: VersionService,
}
