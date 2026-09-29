use std::path::Path;
use rmerge_domain::errors::DomainError;

#[async_trait::async_trait]
pub trait GitCliPort: Send + Sync {
    /// Ejecuta un comando Git subyacente. Lanza DomainError::ProcessError u otros en caso de falla.
    async fn execute(&self, repo_path: &Path, args: &[&str]) -> Result<String, DomainError>;
}
