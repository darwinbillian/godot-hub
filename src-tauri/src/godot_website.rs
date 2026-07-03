use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::error::Error;

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

pub struct GodotWebsiteClient {
    client: ClientWithMiddleware,
}

impl GodotWebsiteClient {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self { client }
    }

    pub async fn list_versions(&self) -> Result<Vec<VersionDto>, Error> {
        let request = self.client.get(
            "https://raw.githubusercontent.com/godotengine/godot-website/master/_data/versions.yml",
        );
        let response = request.send().await?.error_for_status()?;
        let bytes = response.bytes().await?;
        let versions = yaml_serde::from_slice::<Vec<VersionDto>>(&bytes)?;
        Ok(versions)
    }
}
