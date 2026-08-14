use std::{fmt::Display, str::FromStr};

use thiserror::Error;

use super::{Flavor, FlavorKind, Variant, Version};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Release {
    pub version: Version,
    pub flavor: Flavor,
}

impl Release {
    pub fn new(version: Version, flavor: Flavor) -> Self {
        Self { version, flavor }
    }
}

impl FromStr for Release {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split("-").collect::<Vec<&str>>();
        let (version, flavor) = match parts.as_slice() {
            [version, flavor] => (version, flavor),
            _ => return Err(anyhow::anyhow!(ReleaseError::ParseError(s.to_owned()))),
        };

        let value = Self {
            version: version.parse()?,
            flavor: flavor.parse()?,
        };

        Ok(value)
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.version, self.flavor)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ReleaseError {
    #[error("cannot parse release from '{0}'")]
    ParseError(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseVariant {
    pub version: Version,
    pub flavor: Flavor,
    pub variant: Variant,
}

impl ReleaseVariant {
    pub fn new(version: Version, flavor: Flavor, variant: Variant) -> Self {
        Self {
            version,
            flavor,
            variant,
        }
    }
}

impl Display for ReleaseVariant {
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
