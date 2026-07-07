use std::{
    borrow::Borrow,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use http_cache_reqwest::CacheMode;
use reqwest_middleware::ClientWithMiddleware;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::{application::error::Error, application::event::EventDispatcher};

pub struct DownloadService {
    client: ClientWithMiddleware,
    dir: PathBuf,
}

pub struct Download {
    progress_event: EventDispatcher<DownloadProgressEventArgs>,
    url: String,
    name: String,
}

pub struct DownloadProgress {
    pub downloaded: u64,
    pub size: Option<u64>,
}

pub struct DownloadProgressEventArgs {
    pub downloaded: u64,
    pub size: Option<u64>,
}

impl DownloadService {
    pub fn new(client: ClientWithMiddleware, dir: PathBuf) -> Self {
        Self { client, dir }
    }

    pub async fn download(&self, download: Download) -> Result<PathBuf, Error> {
        let request = self
            .client
            .get(&download.url)
            .with_extension(CacheMode::NoStore);
        let response = request.send().await?.error_for_status()?;
        let size = response.content_length();
        let mut stream = response.bytes_stream();

        tokio::fs::create_dir_all(&self.dir).await?;

        let path = self.dir.join(&download.name);
        let temporary_path = path.with_added_extension("part");
        let mut file = File::create(&temporary_path).await?;

        let mut last_progress = Instant::now();
        let mut progress = DownloadProgress {
            downloaded: 0,
            size,
        };

        download
            .progress_event()
            .invoke(Arc::new(DownloadProgressEventArgs::from(&progress)));

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            progress.downloaded += chunk.len() as u64;
            if last_progress.elapsed() >= Duration::from_millis(500) {
                download
                    .progress_event()
                    .invoke(Arc::new(DownloadProgressEventArgs::from(&progress)));
                last_progress = Instant::now();
            }
        }

        download
            .progress_event()
            .invoke(Arc::new(DownloadProgressEventArgs::from(&progress)));

        file.flush().await?;

        tokio::fs::rename(&temporary_path, &path).await?;

        Ok(path)
    }
}

impl Download {
    pub fn new(url: &str, name: &str) -> Self {
        Self {
            progress_event: EventDispatcher::new(),
            url: url.to_owned(),
            name: name.to_owned(),
        }
    }

    pub fn progress_event(&self) -> &EventDispatcher<DownloadProgressEventArgs> {
        &self.progress_event
    }
}

impl<D> From<D> for DownloadProgress
where
    D: Borrow<DownloadProgressEventArgs>,
{
    fn from(value: D) -> Self {
        let value = value.borrow();
        Self {
            downloaded: value.downloaded,
            size: value.size,
        }
    }
}

impl<D> From<D> for DownloadProgressEventArgs
where
    D: Borrow<DownloadProgress>,
{
    fn from(value: D) -> Self {
        let value = value.borrow();
        Self {
            downloaded: value.downloaded,
            size: value.size,
        }
    }
}
