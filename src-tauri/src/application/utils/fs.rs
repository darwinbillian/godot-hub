use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::Result;
use tokio::fs::File;

pub struct FileGuard {
    file: Option<File>,
    path: Option<PathBuf>,
}

impl FileGuard {
    pub async fn create<P>(path: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let file = File::create(&path).await?;
        Ok(Self {
            path: Some(path),
            file: Some(file),
        })
    }

    pub fn disarm(&mut self) {
        self.path.take();
    }
}

impl Deref for FileGuard {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.file.as_ref().unwrap()
    }
}

impl DerefMut for FileGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file.as_mut().unwrap()
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        drop(self.file.take());

        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}
