use std::{
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use bytes::Bytes;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;

use crate::application::{
    error::Error,
    services::task::{CancellationTokenExt, TaskError},
    utils::fs::FileGuard,
};

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
    pub stream: Pin<Box<dyn Stream<Item = Result<DownloadProgress, TaskError>> + Send>>,
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

impl DownloadService {
    pub fn new(download_provider: Arc<dyn DownloadProvider + Send + Sync>, dir: &Path) -> Self {
        Self {
            download_provider,
            dir: dir.to_owned(),
        }
    }

    pub async fn download(
        &self,
        request: DownloadRequest,
        cancellation_token: CancellationToken,
    ) -> Result<DownloadHandle, TaskError> {
        let name = format!(
            "Godot_v{}-{}_{}",
            request.version, request.flavor, request.slug
        );
        let path = self.dir.join(&name);

        let response = self.download_provider.download(request).await?;
        let stream = self
            .stream(response, path.clone(), cancellation_token)
            .await?;
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
        cancellation_token: CancellationToken,
    ) -> Result<impl Stream<Item = Result<DownloadProgress, TaskError>>, TaskError> {
        let mut response = response;

        cancellation_token.error_if_cancelled()?;

        tokio::fs::create_dir_all(&self.dir).await?;

        let temporary_path = path.with_added_extension("part");
        let mut file = FileGuard::create(&temporary_path).await?;

        let size = response.size;
        let mut downloaded = 0u64;

        let stream = async_stream::try_stream! {
            yield DownloadProgress { downloaded, size, status: DownloadStatus::Starting };

            loop {
                let chunk = tokio::select! {
                    biased;
                    _ = cancellation_token.cancelled() => Err(TaskError::Cancelled),
                    chunk = response.stream.next() => Ok(chunk),
                };

                let chunk = match chunk? {
                    Some(chunk) => chunk?,
                    None => break,
                };

                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                yield DownloadProgress { downloaded, size, status: DownloadStatus::Downloading };
            }

            file.flush().await?;
            tokio::fs::rename(&temporary_path, &path).await?;
            file.disarm();

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
