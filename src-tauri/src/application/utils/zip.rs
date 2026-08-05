use std::{
    fs::File,
    path::{Path, PathBuf},
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

        let result = Self {
            archive: Arc::new(Mutex::new(archive)),
        };

        Ok(result)
    }

    pub async fn extract_unwrapped_root_dir<D>(&self, directory: D) -> Result<()>
    where
        D: AsRef<Path>,
    {
        let directory = directory.as_ref().to_owned();
        let archive = self.archive.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut archive = archive.lock().unwrap();
            archive.extract_unwrapped_root_dir(directory, zip::read::root_dir_common_filter)?;
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

    pub fn root_dir(&self) -> Result<Option<PathBuf>> {
        let archive = self.archive.lock().unwrap();
        let root_dir = archive.root_dir(zip::read::root_dir_common_filter)?;
        Ok(root_dir)
    }
}
