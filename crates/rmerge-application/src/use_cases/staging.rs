use std::path::Path;
use std::sync::Arc;
use rmerge_domain::errors::DomainError;
use crate::ports::r#in::ManageStagingUseCase;
use crate::ports::out::GitStoragePort;

pub struct StagingService {
    git_storage: Arc<dyn GitStoragePort>,
}

impl StagingService {
    pub fn new(git_storage: Arc<dyn GitStoragePort>) -> Self {
        Self { git_storage }
    }
}

#[async_trait::async_trait]
impl ManageStagingUseCase for StagingService {
    async fn stage_hunk(&self, repo_path: &Path, file_path: &Path, hunk_index: usize) -> Result<(), DomainError> {
        self.git_storage.stage_hunk(repo_path, file_path, hunk_index).await
    }

    async fn stage_lines(
        &self,
        repo_path: &Path,
        file_path: &Path,
        hunk_index: usize,
        lines: &[usize],
    ) -> Result<(), DomainError> {
        self.git_storage.stage_lines(repo_path, file_path, hunk_index, lines).await
    }

    async fn unstage_lines(
        &self,
        repo_path: &Path,
        file_path: &Path,
        hunk_index: usize,
        lines: &[usize],
    ) -> Result<(), DomainError> {
        self.git_storage.unstage_lines(repo_path, file_path, hunk_index, lines).await
    }

    async fn stage_all(&self, repo_path: &Path) -> Result<(), DomainError> {
        self.git_storage.stage_all(repo_path).await
    }

    async fn unstage_all(&self, repo_path: &Path) -> Result<(), DomainError> {
        self.git_storage.unstage_all(repo_path).await
    }

    async fn discard_file_changes(&self, repo_path: &Path, file_path: &Path) -> Result<(), DomainError> {
        self.git_storage.discard_changes(repo_path, file_path).await
    }
}
