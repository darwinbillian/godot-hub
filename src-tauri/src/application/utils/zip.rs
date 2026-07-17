use std::{fs::File, path::PathBuf};

use anyhow::Result;
use zip::ZipArchive;

pub async fn extract<P, Q>(path: P, directory: Q) -> Result<()>
where
    P: Into<PathBuf>,
    Q: Into<PathBuf>,
{
    let path = path.into();
    let directory = directory.into();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(directory)?;
        Ok(())
    })
    .await??;

    Ok(())
}
