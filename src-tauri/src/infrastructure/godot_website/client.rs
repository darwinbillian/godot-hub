use anyhow::Result;
use http_cache_reqwest::CacheMode;
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;

use super::dtos::VersionDto;

#[derive(Clone)]
pub struct GodotWebsiteClient {
    client: ClientWithMiddleware,
}

impl GodotWebsiteClient {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self { client }
    }

    pub async fn list_versions(&self) -> Result<Vec<VersionDto>> {
        let request = self.client.get(
            "https://raw.githubusercontent.com/godotengine/godot-website/master/_data/versions.yml",
        );
        let response = request.send().await?.error_for_status()?;
        let bytes = response.bytes().await?;
        let versions = yaml_serde::from_slice::<Vec<VersionDto>>(&bytes)?;
        Ok(versions)
    }

    pub async fn download(
        &self,
        version: &str,
        flavor: &str,
        slug: &str,
        platform: &str,
    ) -> Result<Response> {
        let url = format!(
            "https://downloads.godotengine.org/?version={}&flavor={}&slug={}&platform={}",
            version, flavor, slug, platform
        );
        let request = self.client.get(url).with_extension(CacheMode::NoStore);
        let response = request.send().await?.error_for_status()?;
        Ok(response)
    }
}
