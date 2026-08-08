use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
    str::FromStr,
};

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionFlavorVariant {
    pub version: Version,
    pub flavor: Flavor,
    pub variant: Variant,
}

impl VersionFlavorVariant {
    pub fn new(version: Version, flavor: Flavor, variant: Variant) -> Self {
        Self {
            version,
            flavor,
            variant,
        }
    }
}

impl Display for VersionFlavorVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "Godot {:.2}", self.version)?;
            if self.flavor.kind != FlavorKind::Stable {
                write!(f, " {:#}", self.flavor)?;
            }
            if self.variant != Variant::Standard {
                write!(f, " {:#}", self.variant)?;
            }
        } else {
            write!(f, "{}-{}", self.version, self.flavor)?;
            if self.variant != Variant::Standard {
                write!(f, "-{}", self.variant)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionFlavor {
    pub version: Version,
    pub flavor: Flavor,
}

impl VersionFlavor {
    pub fn new(version: Version, flavor: Flavor) -> Self {
        Self { version, flavor }
    }
}

impl FromStr for VersionFlavor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split("-").collect::<Vec<&str>>();
        let (version, flavor) = match parts.as_slice() {
            [version, flavor] => (version, flavor),
            _ => {
                return Err(anyhow::anyhow!(VersionFlavorError::ParseError(
                    s.to_owned()
                )))
            }
        };

        let value = Self {
            version: version.parse()?,
            flavor: flavor.parse()?,
        };

        Ok(value)
    }
}

impl Display for VersionFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.version, self.flavor)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum VersionFlavorError {
    #[error("cannot parse version flavor from '{0}'")]
    ParseError(String),
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        if f.alternate() {
            write!(f, "{:#}", self.kind)?;
        } else {
            write!(f, "{}", self.kind)?;
            if let Some(number) = self.number {
                write!(f, "{}", number)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        if f.alternate() {
            match self {
                Self::Dev => write!(f, "Dev")?,
                Self::Alpha => write!(f, "Alpha")?,
                Self::Beta => write!(f, "Beta")?,
                Self::Rc => write!(f, "Rc")?,
                Self::Stable => write!(f, "Stable")?,
            };
        } else {
            match self {
                Self::Dev => write!(f, "dev")?,
                Self::Alpha => write!(f, "alpha")?,
                Self::Beta => write!(f, "beta")?,
                Self::Rc => write!(f, "rc")?,
                Self::Stable => write!(f, "stable")?,
            };
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlavorKindFlags(u8);

impl FlavorKindFlags {
    pub const NONE: Self = Self(0);
    pub const DEV: Self = Self(1 << 0);
    pub const ALPHA: Self = Self(1 << 1);
    pub const BETA: Self = Self(1 << 2);
    pub const RC: Self = Self(1 << 3);
    pub const STABLE: Self = Self(1 << 4);
    pub const PRERELEASE: Self = Self(Self::DEV.0 | Self::ALPHA.0 | Self::BETA.0 | Self::RC.0);
    pub const ALL: Self = Self(Self::PRERELEASE.0 | Self::STABLE.0);

    pub fn contains<F>(&self, other: F) -> bool
    where
        F: Into<Self>,
    {
        let other = other.into();
        self.0 & other.0 == other.0
    }
}

impl BitOr for FlavorKindFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FlavorKindFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Default for FlavorKindFlags {
    fn default() -> Self {
        Self::ALL
    }
}

impl From<Flavor> for FlavorKindFlags {
    fn from(value: Flavor) -> Self {
        value.kind.into()
    }
}

impl From<FlavorKind> for FlavorKindFlags {
    fn from(value: FlavorKind) -> Self {
        match value {
            FlavorKind::Dev => Self::DEV,
            FlavorKind::Alpha => Self::ALPHA,
            FlavorKind::Beta => Self::BETA,
            FlavorKind::Rc => Self::RC,
            FlavorKind::Stable => Self::STABLE,
        }
    }
}

impl FromStr for FlavorKindFlags {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Self::NONE;

        for part in s.split(',') {
            let flag = match part {
                "dev" => Self::DEV,
                "alpha" => Self::ALPHA,
                "beta" => Self::BETA,
                "rc" => Self::RC,
                "stable" => Self::STABLE,
                "prerelease" => Self::PRERELEASE,
                "all" => Self::ALL,
                _ => return Err(anyhow::anyhow!(FlavorError::UnknownKind(part.to_owned()))),
            };

            flags |= flag
        }

        Ok(flags)
    }
}

#[derive(Error, Debug)]
pub enum FlavorError {
    #[error("unknown flavor kind '{0}'")]
    UnknownKind(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Variant {
    Standard,
    Mono,
}

impl Variant {
    pub fn iter() -> impl Iterator<Item = Variant> + Clone {
        [Variant::Standard, Variant::Mono].into_iter()
    }
}

impl FromStr for Variant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "standard" => Self::Standard,
            "mono" => Self::Mono,
            _ => return Err(anyhow::anyhow!(VariantError::UnknownKind(s.to_owned()))),
        };

        Ok(kind)
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Self::Standard => write!(f, "Standard")?,
                Self::Mono => write!(f, "Mono")?,
            };
        } else {
            match self {
                Self::Standard => write!(f, "standard")?,
                Self::Mono => write!(f, "mono")?,
            };
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum VariantError {
    #[error("unknown variant kind '{0}'")]
    UnknownKind(String),
}
