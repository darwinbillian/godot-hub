use std::path::PathBuf;

use http_cache_reqwest::CacheMode;
use reqwest_middleware::ClientWithMiddleware;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::error::Error;

pub struct DownloadService {
    pub client: ClientWithMiddleware,
    pub dir: PathBuf,
}

impl DownloadService {
    pub async fn download(&self, url: &str, name: &str) -> Result<PathBuf, Error> {
        let request = self.client.get(url).with_extension(CacheMode::NoStore);
        let response = request.send().await?.error_for_status()?;
        let mut stream = response.bytes_stream();

        tokio::fs::create_dir_all(&self.dir).await?;

        let path = self.dir.join(name);
        let temporary_path = path.with_added_extension("part");
        let mut file = File::create(&temporary_path).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        tokio::fs::rename(&temporary_path, &path).await?;

        Ok(path)
    }
}
