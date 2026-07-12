use crate::application::services::{
    install::InstallService, installation::InstallationService, version::VersionService,
};

pub struct AppState {
    pub install_service: InstallService,
    pub installation_service: InstallationService,
    pub version_service: VersionService,
}
