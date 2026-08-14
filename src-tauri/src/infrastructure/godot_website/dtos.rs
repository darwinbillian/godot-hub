use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    application::interfaces::download_configs::{
        DownloadConfig, DownloadConfigGroup, DownloadConfigOverride, DownloadConfigs,
    },
    domain::models::Release,
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct VersionDto {
    pub name: String,
    pub flavor: String,
    pub release_date: String,
    pub release_notes: String,
    pub featured: Option<String>,
    pub releases: Option<Vec<VersionReleaseDto>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct VersionReleaseDto {
    pub name: String,
    pub release_date: String,
    pub release_notes: String,
    pub release_version: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigsDto {
    pub defaults: HashMap<u32, DownloadConfigGroupDto>,
    pub overrides: Vec<DownloadConfigOverrideDto>,
}

impl TryFrom<DownloadConfigsDto> for DownloadConfigs {
    type Error = anyhow::Error;

    fn try_from(value: DownloadConfigsDto) -> Result<Self, Self::Error> {
        let defaults = value
            .defaults
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect::<HashMap<u32, DownloadConfigGroup>>();

        let overrides = value
            .overrides
            .into_iter()
            .filter_map(|r#override| r#override.try_into().ok())
            .collect::<Vec<DownloadConfigOverride>>();

        let result = Self {
            defaults,
            overrides,
        };

        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigOverrideDto {
    pub version: u32,
    pub range: [String; 2],
    pub config: DownloadConfigGroupDto,
}

impl TryFrom<DownloadConfigOverrideDto> for DownloadConfigOverride {
    type Error = anyhow::Error;

    fn try_from(value: DownloadConfigOverrideDto) -> Result<Self, Self::Error> {
        let [start, end] = value.range;

        let start = start.parse::<Release>()?;
        let end = end.parse::<Release>()?;

        let result = Self {
            version: value.version,
            range: start..=end,
            config: value.config.into(),
        };

        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigGroupDto {
    #[serde(flatten)]
    pub standard: DownloadConfigDto,
    pub mono: Option<DownloadConfigDto>,
}

impl From<DownloadConfigGroupDto> for DownloadConfigGroup {
    fn from(value: DownloadConfigGroupDto) -> Self {
        Self {
            standard: value.standard.into(),
            mono: value.mono.map(Into::into),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigDto {
    pub templates: Option<String>,
    pub editor: Option<HashMap<String, String>>,
    pub extras: Option<HashMap<String, String>>,
}

impl From<DownloadConfigDto> for DownloadConfig {
    fn from(value: DownloadConfigDto) -> Self {
        Self {
            editor: value.editor,
        }
    }
}
