use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;

use crate::domain::models::version::Version;

#[async_trait::async_trait]
pub trait DownloadConfigsProvider {
    async fn get_download_configs(&self) -> Result<Arc<dyn DownloadConfigs + Send + Sync>>;
}

pub trait DownloadConfigs {
    fn get_slug(
        &self,
        version: &str,
        flavor: &str,
        mono: bool,
        platform: &str,
    ) -> Result<String, DownloadConfigsError>;
}

#[derive(Error, Debug)]
pub enum DownloadConfigsError {
    #[error("version '{0}' is not valid")]
    VersionNotValid(String),
    #[error("release '{0}' is not available")]
    ReleaseNotAvailable(Version),
    #[error("release '{0}' is not available for platform '{1}'")]
    ReleaseNotAvailableForPlatform(Version, String),
}
