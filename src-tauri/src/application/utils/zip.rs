use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use zip::ZipArchive;

pub struct ZipFile {
    archive: Arc<Mutex<ZipArchive<File>>>,
}

impl ZipFile {
    pub async fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_owned();

        let archive = tokio::task::spawn_blocking(|| -> Result<ZipArchive<File>> {
            let file = File::open(path)?;
            let archive = ZipArchive::new(file)?;
            Ok(archive)
        })
        .await??;

        Ok(Self {
            archive: Arc::new(Mutex::new(archive)),
        })
    }

    pub async fn extract<D>(&self, directory: D) -> Result<()>
    where
        D: AsRef<Path>,
    {
        let directory = directory.as_ref().to_owned();
        let archive = self.archive.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut archive = archive.lock().unwrap();
            archive.extract(directory)?;
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub fn file_names(&self) -> Vec<String> {
        let archive = self.archive.lock().unwrap();
        archive
            .file_names()
            .map(|file_name| file_name.to_owned())
            .collect()
    }
}
