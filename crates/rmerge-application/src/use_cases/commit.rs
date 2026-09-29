use std::path::Path;
use std::sync::Arc;
use rmerge_domain::errors::DomainError;
use crate::ports::r#in::{CommitChangesUseCase, CommitOutcome};
use crate::ports::out::{GitConfigPort, GitStoragePort};

pub struct CommitService {
    git_storage: Arc<dyn GitStoragePort>,
    git_config: Arc<dyn GitConfigPort>,
}

impl CommitService {
    pub fn new(git_storage: Arc<dyn GitStoragePort>, git_config: Arc<dyn GitConfigPort>) -> Self {
        Self {
            git_storage,
            git_config,
        }
    }
}

#[async_trait::async_trait]
impl CommitChangesUseCase for CommitService {
    async fn execute(&self, repo_path: &Path, message: &str) -> Result<CommitOutcome, DomainError> {
        let trimmed_msg = message.trim();
        if trimmed_msg.is_empty() {
            return Err(DomainError::ValidationError("El mensaje de commit no puede estar vacío.".into()));
        }

        // 1. Verificar si hay cambios en el staging
        let status = self.git_storage.get_status(repo_path).await?;
        if status.staged.is_empty() {
            return Ok(CommitOutcome::EmptyStaging);
        }

        // 2. Verificar identidad del autor
        let author = match self.git_config.get_effective_author(repo_path).await? {
            Some(a) => a,
            None => {
                // Intercepta para solicitar al usuario su identidad (Local o Global)
                return Ok(CommitOutcome::AuthorRequired);
            }
        };

        // 3. Crear el commit
        let hash = self.git_storage.create_commit(repo_path, trimmed_msg, &author).await?;
        Ok(CommitOutcome::Success { commit_hash: hash })
    }
}
