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
