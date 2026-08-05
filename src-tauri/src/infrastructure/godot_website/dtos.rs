use std::collections::HashMap;

use serde::Deserialize;

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
    pub defaults: HashMap<String, DownloadConfigGroupDto>,
    pub overrides: Vec<DownloadConfigOverrideDto>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigOverrideDto {
    pub version: String,
    pub range: Vec<String>,
    pub config: DownloadConfigGroupDto,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigGroupDto {
    #[serde(flatten)]
    pub standard: DownloadConfigDto,
    pub mono: Option<DownloadConfigDto>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigDto {
    pub templates: Option<String>,
    pub editor: Option<HashMap<String, String>>,
    pub extras: Option<HashMap<String, String>>,
}
