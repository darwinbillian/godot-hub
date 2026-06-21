use reqwest_middleware::ClientWithMiddleware;

use crate::{error::Error, godot_website::Version};

pub struct VersionService {
    pub client: ClientWithMiddleware,
}

impl VersionService {
    pub async fn list(&self) -> Result<Vec<Version>, Error> {
        let versions = crate::godot_website::get_versions(&self.client).await?;
        Ok(versions)
    }
}
