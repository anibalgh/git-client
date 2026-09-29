use rmerge_domain::entities::{Branch, CommitDetail, CommitGraph, MergeOutcome, StagingArea, Tag};
use rmerge_domain::errors::DomainError;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RepositoryData {
    pub staging: StagingArea,
    pub graph: CommitGraph,
    pub branches: Vec<Branch>,
    pub tags: Vec<Tag>,
    pub current_branch: Option<String>,
}

#[async_trait::async_trait]
pub trait LoadRepositoryUseCase: Send + Sync {
    async fn execute(&self, repo_path: &Path) -> Result<RepositoryData, DomainError>;
}

#[async_trait::async_trait]
pub trait CloneRepositoryUseCase: Send + Sync {
    async fn clone_repo(&self, url: &str, destination: &Path) -> Result<(), DomainError>;
}

#[async_trait::async_trait]
pub trait ManageBranchesUseCase: Send + Sync {
    async fn create_branch(&self, repo_path: &Path, name: &str, start_point: Option<&str>, checkout: bool) -> Result<(), DomainError>;
    async fn switch_branch(&self, repo_path: &Path, name: &str) -> Result<(), DomainError>;
    async fn merge_branch(&self, repo_path: &Path, source_branch: &str) -> Result<MergeOutcome, DomainError>;
}

#[async_trait::async_trait]
pub trait ManageTagsUseCase: Send + Sync {
    async fn get_tags(&self, repo_path: &Path) -> Result<Vec<Tag>, DomainError>;
    async fn create_tag(&self, repo_path: &Path, name: &str, target_commit: Option<&str>, message: Option<&str>) -> Result<(), DomainError>;
    async fn delete_tag(&self, repo_path: &Path, name: &str) -> Result<(), DomainError>;
}

#[async_trait::async_trait]
pub trait InspectCommitUseCase: Send + Sync {
    async fn get_commit_detail(&self, repo_path: &Path, commit_id: &str) -> Result<CommitDetail, DomainError>;
}

#[async_trait::async_trait]
pub trait RemoteUrlUseCase: Send + Sync {
    async fn get_remote_url(&self, repo_path: &Path) -> Result<Option<String>, DomainError>;
}

#[async_trait::async_trait]
pub trait ManageGitignoreUseCase: Send + Sync {
    async fn get_gitignore(&self, repo_path: &Path) -> Result<Option<String>, DomainError>;
    async fn save_gitignore(&self, repo_path: &Path, content: &str) -> Result<(), DomainError>;
}
