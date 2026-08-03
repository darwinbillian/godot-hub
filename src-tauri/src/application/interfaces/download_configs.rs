use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;

use crate::domain::models::version::{Flavor, Version};

#[async_trait::async_trait]
pub trait DownloadConfigsProvider {
    async fn get_download_configs(&self) -> Result<Arc<dyn DownloadConfigs + Send + Sync>>;
}

pub trait DownloadConfigs {
    fn get_slug(
        &self,
        version: Version,
        flavor: Flavor,
        mono: bool,
        platform: &str,
    ) -> Result<String, DownloadConfigsError>;
}

#[derive(Error, Debug)]
pub enum DownloadConfigsError {
    #[error("release '{0}-{1}' is not available")]
    ReleaseNotAvailable(Version, Flavor),
    #[error("release '{0}-{1}' is not available for platform '{2}'")]
    ReleaseNotAvailableForPlatform(Version, Flavor, String),
}
