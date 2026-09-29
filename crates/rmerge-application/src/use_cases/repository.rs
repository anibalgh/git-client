use std::path::Path;
use std::sync::Arc;
use rmerge_domain::entities::{CommitDetail, MergeOutcome, Tag};
use rmerge_domain::errors::DomainError;
use rmerge_domain::services::CommitGraphCalculator;
use crate::ports::r#in::{
    CloneRepositoryUseCase, InspectCommitUseCase, LoadRepositoryUseCase, ManageBranchesUseCase,
    ManageGitignoreUseCase, ManageTagsUseCase, RemoteUrlUseCase, RepositoryData,
};
use crate::ports::out::GitStoragePort;

pub struct RepositoryService {
    git_storage: Arc<dyn GitStoragePort>,
}

impl RepositoryService {
    pub fn new(git_storage: Arc<dyn GitStoragePort>) -> Self {
        Self { git_storage }
    }
}

#[async_trait::async_trait]
impl LoadRepositoryUseCase for RepositoryService {
    async fn execute(&self, repo_path: &Path) -> Result<RepositoryData, DomainError> {
        let staging = self.git_storage.get_status(repo_path).await?;
        let raw_commits = self.git_storage.get_commit_history(repo_path, 200).await?;
        let branches = self.git_storage.get_branches(repo_path).await?;
        let tags = self.git_storage.get_tags(repo_path).await.unwrap_or_default();

        let current_branch = branches
            .iter()
            .find(|b| b.is_head)
            .map(|b| b.name.clone());

        // Calcular carriles del grafo de commits usando el servicio de dominio
        let graph = CommitGraphCalculator::compute_lanes(raw_commits);

        Ok(RepositoryData {
            staging,
            graph,
            branches,
            tags,
            current_branch,
        })
    }
}

#[async_trait::async_trait]
impl CloneRepositoryUseCase for RepositoryService {
    async fn clone_repo(&self, url: &str, destination: &Path) -> Result<(), DomainError> {
        self.git_storage.clone_repository(url, destination).await
    }
}

#[async_trait::async_trait]
impl ManageBranchesUseCase for RepositoryService {
    async fn create_branch(&self, repo_path: &Path, name: &str, start_point: Option<&str>, checkout: bool) -> Result<(), DomainError> {
        self.git_storage.create_branch(repo_path, name, start_point, checkout).await
    }

    async fn switch_branch(&self, repo_path: &Path, name: &str) -> Result<(), DomainError> {
        self.git_storage.switch_branch(repo_path, name).await
    }

    async fn merge_branch(&self, repo_path: &Path, source_branch: &str) -> Result<MergeOutcome, DomainError> {
        self.git_storage.merge_branch(repo_path, source_branch).await
    }
}

#[async_trait::async_trait]
impl ManageTagsUseCase for RepositoryService {
    async fn get_tags(&self, repo_path: &Path) -> Result<Vec<Tag>, DomainError> {
        self.git_storage.get_tags(repo_path).await
    }

    async fn create_tag(&self, repo_path: &Path, name: &str, target_commit: Option<&str>, message: Option<&str>) -> Result<(), DomainError> {
        self.git_storage.create_tag(repo_path, name, target_commit, message).await
    }

    async fn delete_tag(&self, repo_path: &Path, name: &str) -> Result<(), DomainError> {
        self.git_storage.delete_tag(repo_path, name).await
    }
}

#[async_trait::async_trait]
impl InspectCommitUseCase for RepositoryService {
    async fn get_commit_detail(&self, repo_path: &Path, commit_id: &str) -> Result<CommitDetail, DomainError> {
        self.git_storage.get_commit_detail(repo_path, commit_id).await
    }
}

#[async_trait::async_trait]
impl RemoteUrlUseCase for RepositoryService {
    async fn get_remote_url(&self, repo_path: &Path) -> Result<Option<String>, DomainError> {
        self.git_storage.get_remote_url(repo_path, None).await
    }
}

#[async_trait::async_trait]
impl ManageGitignoreUseCase for RepositoryService {
    async fn get_gitignore(&self, repo_path: &Path) -> Result<Option<String>, DomainError> {
        self.git_storage.get_gitignore(repo_path).await
    }

    async fn save_gitignore(&self, repo_path: &Path, content: &str) -> Result<(), DomainError> {
        self.git_storage.save_gitignore(repo_path, content).await
    }
}
