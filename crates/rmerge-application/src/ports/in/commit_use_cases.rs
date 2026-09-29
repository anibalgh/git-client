use rmerge_domain::errors::DomainError;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutcome {
    Success { commit_hash: String },
    AuthorRequired,
    EmptyStaging,
}

#[async_trait::async_trait]
pub trait CommitChangesUseCase: Send + Sync {
    async fn execute(&self, repo_path: &Path, message: &str) -> Result<CommitOutcome, DomainError>;
}
