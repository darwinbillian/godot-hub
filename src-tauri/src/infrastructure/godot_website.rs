use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::application::{
    error::Error,
    services::{
        install::DownloadProvider,
        version::{RemoteVersion, VersionProvider},
    },
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

pub struct GodotWebsiteClient {
    client: ClientWithMiddleware,
}

pub struct GodotWebsiteVersionProvider {
    client: GodotWebsiteClient,
}

pub struct GodotWebsiteDownloadProvider;

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

impl GodotWebsiteVersionProvider {
    pub fn new(client: GodotWebsiteClient) -> Self {
        Self { client }
    }
}

impl GodotWebsiteDownloadProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VersionProvider for GodotWebsiteVersionProvider {
    async fn list_versions(&self) -> Result<Vec<RemoteVersion>, Error> {
        let versions = self.client.list_versions().await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| RemoteVersion {
                name: version.name,
                flavor: version.flavor,
                release_notes: format!(
                    "https://godotengine.org/{}",
                    version.release_notes.trim_start_matches("/")
                ),
            })
            .collect())
    }
}

impl DownloadProvider for GodotWebsiteDownloadProvider {
    fn get_download_url(&self, version: &str, flavor: &str, slug: &str, platform: &str) -> String {
        format!(
            "https://downloads.godotengine.org/?version={}&flavor={}&slug={}&platform={}",
            version, flavor, slug, platform
        )
    }
}
