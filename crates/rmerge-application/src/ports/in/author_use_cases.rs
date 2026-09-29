use rmerge_domain::entities::{ConfigScope, GitAuthor};
use rmerge_domain::errors::DomainError;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorStatus {
    Configured(GitAuthor),
    Missing {
        suggested_name: Option<String>,
        suggested_email: Option<String>,
    },
}

#[async_trait::async_trait]
pub trait EnsureAuthorUseCase: Send + Sync {
    /// Comprueba si la identidad de autor existe para el repositorio dado
    async fn check_author(&self, repo_path: &Path) -> Result<AuthorStatus, DomainError>;
}

#[async_trait::async_trait]
pub trait ConfigureAuthorUseCase: Send + Sync {
    /// Guarda la identidad del autor en el ámbito especificado (Local o Global) tras validar su formato
    async fn configure_author(
        &self,
        repo_path: &Path,
        author: GitAuthor,
        scope: ConfigScope,
    ) -> Result<(), DomainError>;
}
