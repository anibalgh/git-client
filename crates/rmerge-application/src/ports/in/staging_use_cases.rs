use rmerge_domain::errors::DomainError;
use std::path::Path;

#[async_trait::async_trait]
pub trait ManageStagingUseCase: Send + Sync {
    async fn stage_hunk(&self, repo_path: &Path, file_path: &Path, hunk_index: usize) -> Result<(), DomainError>;
    async fn stage_lines(&self, repo_path: &Path, file_path: &Path, hunk_index: usize, lines: &[usize]) -> Result<(), DomainError>;
    async fn unstage_lines(&self, repo_path: &Path, file_path: &Path, hunk_index: usize, lines: &[usize]) -> Result<(), DomainError>;
    async fn stage_all(&self, repo_path: &Path) -> Result<(), DomainError>;
    async fn unstage_all(&self, repo_path: &Path) -> Result<(), DomainError>;
    async fn discard_file_changes(&self, repo_path: &Path, file_path: &Path) -> Result<(), DomainError>;
}
