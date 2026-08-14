use std::pin::Pin;

use anyhow::Result;
use bytes::Bytes;
use tokio_stream::Stream;

use crate::domain::models::{Flavor, Version};

#[async_trait::async_trait]
pub trait DownloadProvider {
    async fn download(&self, download: DownloadRequest) -> Result<DownloadResponse>;
}

pub struct DownloadRequest {
    pub version: Version,
    pub flavor: Flavor,
    pub slug: String,
    pub platform: String,
}

impl DownloadRequest {
    pub fn new(version: Version, flavor: Flavor, slug: &str, platform: &str) -> Self {
        Self {
            version,
            flavor,
            slug: slug.to_owned(),
            platform: platform.to_owned(),
        }
    }
}

pub struct DownloadResponse {
    pub stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>,
    pub size: Option<u64>,
}
