use rmerge_domain::entities::{ConfigScope, GitAuthor};
use rmerge_domain::errors::DomainError;
use std::path::Path;

#[async_trait::async_trait]
pub trait GitConfigPort: Send + Sync {
    /// Obtiene la identidad efectiva del autor resolviendo Local -> Global -> System
    async fn get_effective_author(&self, repo_path: &Path) -> Result<Option<GitAuthor>, DomainError>;

    /// Obtiene la identidad definida estrictamente en el ámbito local (.git/config)
    async fn get_local_author(&self, repo_path: &Path) -> Result<Option<GitAuthor>, DomainError>;

    /// Obtiene la identidad definida estrictamente en el ámbito global (~/.gitconfig)
    async fn get_global_author(&self) -> Result<Option<GitAuthor>, DomainError>;

    /// Guarda la identidad en el archivo de configuración correspondiente según el ámbito seleccionado
    async fn set_author(
        &self,
        repo_path: &Path,
        author: &GitAuthor,
        scope: ConfigScope,
    ) -> Result<(), DomainError>;
}
