use std::pin::Pin;

use anyhow::Result;
use bytes::Bytes;
use tokio_stream::Stream;

#[async_trait::async_trait]
pub trait DownloadProvider {
    async fn download(&self, download: DownloadRequest) -> Result<DownloadResponse>;
}

pub struct DownloadRequest {
    pub version: String,
    pub flavor: String,
    pub slug: String,
    pub platform: String,
}

impl DownloadRequest {
    pub fn new(version: &str, flavor: &str, slug: &str, platform: &str) -> Self {
        Self {
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            slug: slug.to_owned(),
            platform: platform.to_owned(),
        }
    }
}

pub struct DownloadResponse {
    pub stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>,
    pub size: Option<u64>,
}
