use std::path::Path;
use std::sync::Arc;
use rmerge_domain::entities::{ConfigScope, GitAuthor};
use rmerge_domain::errors::DomainError;
use rmerge_domain::services::AuthorValidator;
use crate::ports::r#in::{AuthorStatus, ConfigureAuthorUseCase, EnsureAuthorUseCase};
use crate::ports::out::GitConfigPort;

pub struct AuthorService {
    git_config: Arc<dyn GitConfigPort>,
}

impl AuthorService {
    pub fn new(git_config: Arc<dyn GitConfigPort>) -> Self {
        Self { git_config }
    }
}

#[async_trait::async_trait]
impl EnsureAuthorUseCase for AuthorService {
    async fn check_author(&self, repo_path: &Path) -> Result<AuthorStatus, DomainError> {
        let effective = self.git_config.get_effective_author(repo_path).await?;
        match effective {
            Some(author) => Ok(AuthorStatus::Configured(author)),
            None => {
                // Sugerir nombres de variables de entorno si existen (USER, USERNAME)
                let user_env = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).ok();
                Ok(AuthorStatus::Missing {
                    suggested_name: user_env,
                    suggested_email: None,
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl ConfigureAuthorUseCase for AuthorService {
    async fn configure_author(
        &self,
        repo_path: &Path,
        author: GitAuthor,
        scope: ConfigScope,
    ) -> Result<(), DomainError> {
        // Regla de dominio: validar nombre y correo antes de persistir
        AuthorValidator::validate(&author)?;
        self.git_config.set_author(repo_path, &author, scope).await
    }
}
