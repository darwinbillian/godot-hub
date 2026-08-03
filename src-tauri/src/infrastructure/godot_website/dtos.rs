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
    pub defaults: HashMap<String, DownloadConfigsItemWithMonoDto>,
    pub overrides: Vec<DownloadConfigsOverrideDto>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigsOverrideDto {
    pub version: String,
    pub range: Vec<String>,
    pub config: DownloadConfigsItemWithMonoDto,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigsItemWithMonoDto {
    pub templates: Option<String>,
    pub editor: Option<HashMap<String, String>>,
    pub extras: Option<HashMap<String, String>>,
    pub mono: Option<DownloadConfigsItemDto>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DownloadConfigsItemDto {
    pub templates: Option<String>,
    pub editor: Option<HashMap<String, String>>,
    pub extras: Option<HashMap<String, String>>,
}
