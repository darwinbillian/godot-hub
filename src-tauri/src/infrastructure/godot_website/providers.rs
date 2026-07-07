use super::client::GodotWebsiteClient;
use crate::application::{
    error::Error,
    services::{
        install::DownloadProvider,
        version::{RemoteVersion, VersionProvider},
    },
};

pub struct GodotWebsiteVersionProvider {
    client: GodotWebsiteClient,
}

pub struct GodotWebsiteDownloadProvider;

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
