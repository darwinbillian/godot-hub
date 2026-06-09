use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_date: String,
    pub release_notes: String,
    pub featured: Option<String>,
    pub releases: Option<Vec<VersionRelease>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionRelease {
    pub name: String,
    pub release_date: String,
    pub release_notes: String,
    pub release_version: Option<String>,
}

pub async fn get_versions(client: &ClientWithMiddleware) -> Result<Vec<Version>, Error> {
    let request = client.get(
        "https://raw.githubusercontent.com/godotengine/godot-website/master/_data/versions.yml",
    );
    let response = request.send().await?.error_for_status()?;
    let bytes = response.bytes().await?;
    let versions = yaml_serde::from_slice::<Vec<Version>>(&bytes)?;
    Ok(versions)
}
