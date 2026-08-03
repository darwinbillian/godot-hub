use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Flavor {
    pub kind: FlavorKind,
    pub number: Option<u32>,
}

impl FromStr for Flavor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.find(|c: char| !c.is_ascii_lowercase()).unwrap_or(s.len());
        let (kind, number) = s.split_at(split);

        let flavor = Self {
            kind: kind.parse()?,
            number: (!number.is_empty()).then(|| number.parse()).transpose()?,
        };

        Ok(flavor)
    }
}

impl Display for Flavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(number) = self.number {
            write!(f, "{}", number)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlavorKind {
    Dev,
    Alpha,
    Beta,
    Rc,
    Stable,
}

impl FromStr for FlavorKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "dev" => Self::Dev,
            "alpha" => Self::Alpha,
            "beta" => Self::Beta,
            "rc" => Self::Rc,
            "stable" => Self::Stable,
            _ => return Err(anyhow::anyhow!(FlavorError::UnknownKind(s.to_owned()))),
        };

        Ok(kind)
    }
}

impl Display for FlavorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::Dev => "dev",
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Rc => "rc",
            Self::Stable => "stable",
        };

        write!(f, "{}", kind)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum FlavorError {
    #[error("unknown flavor kind '{0}'")]
    UnknownKind(String),
}
