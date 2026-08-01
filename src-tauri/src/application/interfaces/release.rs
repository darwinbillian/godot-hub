use anyhow::Result;

#[async_trait::async_trait]
pub trait ReleaseProvider {
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>>;
}

pub struct ReleaseMetadata {
    pub name: String,
    pub flavor: String,
    pub release_notes: String,
}
