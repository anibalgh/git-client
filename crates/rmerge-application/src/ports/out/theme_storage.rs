use rmerge_domain::errors::DomainError;
use rmerge_domain::value_objects::ThemeConfig;

#[async_trait::async_trait]
pub trait ThemeStoragePort: Send + Sync {
    async fn list_available_themes(&self) -> Result<Vec<String>, DomainError>;
    async fn get_theme(&self, name: &str) -> Result<ThemeConfig, DomainError>;
}
