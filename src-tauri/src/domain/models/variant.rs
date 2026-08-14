use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
    str::FromStr,
};

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Variant {
    Standard,
    Mono,
}

impl Variant {
    pub fn iter() -> impl Iterator<Item = Variant> {
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

#[derive(Clone, Copy, Debug)]
pub struct VariantFlags(u8);

impl VariantFlags {
    pub const NONE: Self = Self(0);
    pub const STANDARD: Self = Self(1 << 0);
    pub const MONO: Self = Self(1 << 1);
    pub const ALL: Self = Self(Self::STANDARD.0 | Self::MONO.0);

    pub fn contains<V>(&self, other: V) -> bool
    where
        V: Into<Self>,
    {
        let other = other.into();
        self.0 & other.0 == other.0
    }
}

impl BitOr for VariantFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for VariantFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Default for VariantFlags {
    fn default() -> Self {
        Self::ALL
    }
}

impl From<Variant> for VariantFlags {
    fn from(value: Variant) -> Self {
        match value {
            Variant::Standard => Self::STANDARD,
            Variant::Mono => Self::MONO,
        }
    }
}

impl FromStr for VariantFlags {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Self::NONE;

        for part in s.split(',') {
            flags |= match part {
                "standard" => Self::STANDARD,
                "mono" => Self::MONO,
                "all" => Self::ALL,
                _ => return Err(anyhow::anyhow!(VariantError::UnknownKind(part.to_owned()))),
            }
        }

        Ok(flags)
    }
}

#[derive(Error, Debug)]
pub enum VariantError {
    #[error("unknown variant kind '{0}'")]
    UnknownKind(String),
}
