use std::sync::Arc;

use anyhow::Result;

#[async_trait::async_trait]
pub trait DownloadConfigsProvider {
    async fn get_download_configs(&self) -> Result<Arc<dyn DownloadConfigs>>;
}

pub trait DownloadConfigs {
    fn get_slug(&self, version: &str, flavor: &str, platform: &str) -> Result<String>;
}
