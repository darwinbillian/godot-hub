use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;

use crate::domain::models::version::{Flavor, Variant, Version, VersionFlavorVariant};

#[async_trait::async_trait]
pub trait DownloadConfigsProvider {
    async fn get_download_configs(&self) -> Result<Arc<dyn DownloadConfigs + Send + Sync>>;
}

pub trait DownloadConfigs {
    fn get_slug(
        &self,
        version: Version,
        flavor: Flavor,
        variant: Variant,
        platform: &str,
    ) -> Result<String, DownloadConfigsError>;
}

#[derive(Error, Debug)]
pub enum DownloadConfigsError {
    #[error("release '{0}' is not available")]
    ReleaseNotAvailable(VersionFlavorVariant),
    #[error("release '{0}' is not available for platform '{1}'")]
    ReleaseNotAvailableForPlatform(VersionFlavorVariant, String),
}
