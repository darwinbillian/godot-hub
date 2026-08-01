use anyhow::Result;
use thiserror::Error;

#[derive(Clone)]
pub struct PlatformService;

impl PlatformService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_platform(&self) -> Result<String> {
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "linux.64",
            ("windows", "x86_64") => "windows.64",
            (os, arch) => {
                return Err(anyhow::anyhow!(PlatformError::PlatformNotSupported {
                    arch: arch.to_owned(),
                    os: os.to_owned(),
                }))
            }
        };

        Ok(platform.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("platform '{os}-{arch}' is not supported")]
    PlatformNotSupported { arch: String, os: String },
}
