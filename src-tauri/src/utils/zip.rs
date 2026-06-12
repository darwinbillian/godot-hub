use std::{fs::File, path::PathBuf};

use zip::ZipArchive;

use crate::error::Error;

pub async fn extract<P, Q>(path: P, directory: Q) -> Result<(), Error>
where
    P: Into<PathBuf>,
    Q: Into<PathBuf>,
{
    let path = path.into();
    let directory = directory.into();

    tokio::task::spawn_blocking(move || -> Result<(), Error> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(directory)?;
        Ok(())
    })
    .await??;

    Ok(())
}
