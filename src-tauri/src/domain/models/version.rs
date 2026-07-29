use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(".").collect::<Vec<&str>>();
        let (major, minor, patch) = match parts.as_slice() {
            [major, minor] => (major, minor, None),
            [major, minor, patch] => (major, minor, Some(patch)),
            _ => return Err(anyhow::anyhow!(VersionError::ParseError(s.to_owned()))),
        };

        let version = Version {
            major: major.parse()?,
            minor: minor.parse()?,
            patch: patch.map(|patch| patch.parse()).transpose()?,
        };

        Ok(version)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if let Some(patch) = self.patch {
            write!(f, ".{}", patch)?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("cannot parse version from '{0}'")]
    ParseError(String),
}
