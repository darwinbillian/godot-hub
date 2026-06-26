use reqwest_middleware::ClientWithMiddleware;

use crate::error::Error;

pub struct VersionService {
    client: ClientWithMiddleware,
}

pub struct Version {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
}

impl VersionService {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = crate::godot_website::get_versions(&self.client).await?;
        Ok(versions
            .into_iter()
            .filter(|version| version.flavor == "stable")
            .map(|version| Version {
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
