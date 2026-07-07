use reqwest_middleware::ClientWithMiddleware;

use super::dtos::VersionDto;
use crate::application::error::Error;

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
