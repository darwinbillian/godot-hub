use anyhow::Result;

use crate::domain::models::{Flavor, Version};

#[async_trait::async_trait]
pub trait ReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>>;
}

pub struct ReleaseMetadata {
    pub version: Version,
    pub flavor: Flavor,
    pub release_notes: String,
}
