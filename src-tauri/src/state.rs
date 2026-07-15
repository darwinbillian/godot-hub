use crate::application::services::{
    install::InstallService, installation::InstallationService, release::ReleaseService,
};

pub struct AppState {
    pub install_service: InstallService,
    pub installation_service: InstallationService,
    pub release_service: ReleaseService,
}
