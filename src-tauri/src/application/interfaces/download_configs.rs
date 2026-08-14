use std::{collections::HashMap, ops::RangeInclusive};

use anyhow::Result;
use thiserror::Error;

use crate::domain::models::{Flavor, Release, ReleaseVariant, Variant, Version};

#[async_trait::async_trait]
pub trait DownloadConfigsProvider {
    async fn get_download_configs(&self) -> Result<DownloadConfigs>;
}

pub struct DownloadConfigs {
    pub defaults: HashMap<u32, DownloadConfigGroup>,
    pub overrides: Vec<DownloadConfigOverride>,
}

impl DownloadConfigs {
    pub fn get_slug(
        &self,
        version: Version,
        flavor: Flavor,
        variant: Variant,
        platform: &str,
    ) -> Result<String, DownloadConfigsError> {
        let group = self
            .overrides
            .iter()
            .rfind(|r#override| {
                r#override.version == version.major
                    && r#override.range.contains(&Release::new(version, flavor))
            })
            .map(|r#override| &r#override.config)
            .or_else(|| self.defaults.get(&version.major))
            .ok_or_else(|| {
                DownloadConfigsError::ReleaseNotAvailable(ReleaseVariant::new(
                    version, flavor, variant,
                ))
            })?;

        let download_config = match variant {
            Variant::Standard => &group.standard,
            Variant::Mono => group.mono.as_ref().ok_or_else(|| {
                DownloadConfigsError::ReleaseNotAvailable(ReleaseVariant::new(
                    version, flavor, variant,
                ))
            })?,
        };

        let editor = download_config.editor.as_ref().ok_or_else(|| {
            DownloadConfigsError::ReleaseNotAvailable(ReleaseVariant::new(version, flavor, variant))
        })?;

        let slug = editor.get(platform).ok_or_else(|| {
            DownloadConfigsError::ReleaseNotAvailableForPlatform(
                ReleaseVariant::new(version, flavor, variant),
                platform.to_owned(),
            )
        })?;

        Ok(slug.to_owned())
    }
}

pub struct DownloadConfigOverride {
    pub version: u32,
    pub range: RangeInclusive<Release>,
    pub config: DownloadConfigGroup,
}

pub struct DownloadConfigGroup {
    pub standard: DownloadConfig,
    pub mono: Option<DownloadConfig>,
}

pub struct DownloadConfig {
    pub editor: Option<HashMap<String, String>>,
}

#[derive(Error, Debug)]
pub enum DownloadConfigsError {
    #[error("release '{0}' is not available")]
    ReleaseNotAvailable(ReleaseVariant),
    #[error("release '{0}' is not available for platform '{1}'")]
    ReleaseNotAvailableForPlatform(ReleaseVariant, String),
}
