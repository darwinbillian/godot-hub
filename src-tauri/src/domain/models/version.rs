use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
    pub build: Option<u32>,
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(".").collect::<Vec<&str>>();
        let (major, minor, patch, build) = match parts.as_slice() {
            [major, minor] => (major, minor, None, None),
            [major, minor, patch] => (major, minor, Some(patch), None),
            [major, minor, patch, build] => (major, minor, Some(patch), Some(build)),
            _ => return Err(anyhow::anyhow!(VersionError::ParseError(s.to_owned()))),
        };

        let version = Self {
            major: major.parse()?,
            minor: minor.parse()?,
            patch: patch.map(|patch| patch.parse()).transpose()?,
            build: build.map(|build| build.parse()).transpose()?,
        };

        Ok(version)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision();

        match precision {
            None => write!(f, "{}", self.major)?,
            Some(precision) if precision >= 1 => write!(f, "{}", self.major)?,
            _ => return Ok(()),
        }

        match precision {
            None => write!(f, ".{}", self.minor)?,
            Some(precision) if precision >= 2 => write!(f, ".{}", self.minor)?,
            _ => return Ok(()),
        }

        match (precision, self.patch) {
            (None, Some(patch)) => write!(f, ".{}", patch)?,
            (Some(precision), patch) if precision >= 3 => write!(f, ".{}", patch.unwrap_or(0))?,
            _ => return Ok(()),
        }

        match (precision, self.build) {
            (None, Some(build)) => write!(f, ".{}", build)?,
            (Some(precision), build) if precision >= 4 => write!(f, ".{}", build.unwrap_or(0))?,
            _ => return Ok(()),
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("cannot parse version from '{0}'")]
    ParseError(String),
}
