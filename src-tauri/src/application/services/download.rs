use std::{path::PathBuf, pin::Pin, sync::Arc};

use bytes::Bytes;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::{Stream, StreamExt};

use crate::application::error::Error;

#[async_trait::async_trait]
pub trait DownloadProvider {
    async fn download(&self, download: DownloadRequest) -> Result<DownloadResponse, Error>;
}

pub struct DownloadService {
    download_provider: Arc<dyn DownloadProvider + Send + Sync>,
    dir: PathBuf,
}

pub struct DownloadRequest {
    pub version: String,
    pub flavor: String,
    pub slug: String,
    pub platform: String,
}

pub struct DownloadResponse {
    pub stream: Pin<Box<dyn Stream<Item = Result<Bytes, Error>> + Send>>,
    pub size: Option<u64>,
}

pub struct DownloadHandle {
    pub stream: Pin<Box<dyn Stream<Item = Result<DownloadProgress, Error>> + Send>>,
    pub path: PathBuf,
}

pub struct DownloadProgress {
    pub downloaded: u64,
    pub size: Option<u64>,
    pub status: DownloadStatus,
}

#[derive(PartialEq)]
pub enum DownloadStatus {
    Starting,
    Downloading,
    Completed,
}

pub struct DownloadGuard {}

impl DownloadService {
    pub fn new(download_provider: Arc<dyn DownloadProvider + Send + Sync>, dir: PathBuf) -> Self {
        Self {
            download_provider,
            dir,
        }
    }

    pub async fn download(&self, request: DownloadRequest) -> Result<DownloadHandle, Error> {
        let name = format!(
            "Godot_v{}-{}_{}",
            request.version, request.flavor, request.slug
        );
        let path = self.dir.join(&name);

        let response = self.download_provider.download(request).await?;
        let stream = self.stream(response, path.clone()).await?;
        let handle = DownloadHandle {
            stream: Box::pin(stream),
            path,
        };

        Ok(handle)
    }

    async fn stream(
        &self,
        response: DownloadResponse,
        path: PathBuf,
    ) -> Result<impl Stream<Item = Result<DownloadProgress, Error>>, Error> {
        let mut response = response;

        tokio::fs::create_dir_all(&self.dir).await?;

        let temporary_path = path.with_added_extension("part");
        let mut file = File::create(&temporary_path).await?;

        let size = response.size;
        let mut downloaded = 0u64;

        let stream = async_stream::try_stream! {
            yield DownloadProgress { downloaded, size, status: DownloadStatus::Starting };

            while let Some(chunk) = response.stream.try_next().await? {
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                yield DownloadProgress { downloaded, size, status: DownloadStatus::Downloading };
            }

            file.flush().await?;
            tokio::fs::rename(&temporary_path, &path).await?;

            yield DownloadProgress { downloaded, size, status: DownloadStatus::Completed };
        };

        Ok(stream)
    }
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
