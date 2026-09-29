#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use rmerge_domain::entities::{Branch, Commit, ConfigScope, DeltaKind, FilePatch, GitAuthor, StagingArea};
    use rmerge_domain::errors::DomainError;
    use crate::ports::r#in::{CloneRepositoryUseCase, CommitChangesUseCase, CommitOutcome, ConfigureAuthorUseCase};
    use crate::ports::out::{GitConfigPort, GitStoragePort};
    use crate::use_cases::author::AuthorService;
    use crate::use_cases::commit::CommitService;
    use crate::use_cases::repository::RepositoryService;

    struct MockGitConfig {
        author: Mutex<Option<GitAuthor>>,
        last_scope: Mutex<Option<ConfigScope>>,
    }

    #[async_trait::async_trait]
    impl GitConfigPort for MockGitConfig {
        async fn get_effective_author(&self, _repo: &Path) -> Result<Option<GitAuthor>, DomainError> {
            Ok(self.author.lock().unwrap().clone())
        }

        async fn get_local_author(&self, _repo: &Path) -> Result<Option<GitAuthor>, DomainError> {
            Ok(self.author.lock().unwrap().clone())
        }

        async fn get_global_author(&self) -> Result<Option<GitAuthor>, DomainError> {
            Ok(self.author.lock().unwrap().clone())
        }

        async fn set_author(&self, _repo: &Path, author: &GitAuthor, scope: ConfigScope) -> Result<(), DomainError> {
            *self.author.lock().unwrap() = Some(author.clone());
            *self.last_scope.lock().unwrap() = Some(scope);
            Ok(())
        }
    }

    struct MockGitStorage {
        staged: bool,
    }

    #[async_trait::async_trait]
    impl GitStoragePort for MockGitStorage {
        async fn get_status(&self, _repo: &Path) -> Result<StagingArea, DomainError> {
            let mut area = StagingArea::default();
            if self.staged {
                area.staged.push(FilePatch {
                    path: "file.txt".into(),
                    old_path: None,
                    status: DeltaKind::Modified,
                    is_staged: true,
                    hunks: vec![],
                    additions: 1,
                    deletions: 0,
                });
            }
            Ok(area)
        }

        async fn get_commit_history(&self, _repo: &Path, _limit: usize) -> Result<Vec<Commit>, DomainError> {
            Ok(vec![])
        }

        async fn get_branches(&self, _repo: &Path) -> Result<Vec<Branch>, DomainError> {
            Ok(vec![])
        }

        async fn stage_hunk(&self, _repo: &Path, _file: &Path, _idx: usize) -> Result<(), DomainError> {
            Ok(())
        }

        async fn stage_lines(&self, _repo: &Path, _file: &Path, _idx: usize, _lines: &[usize]) -> Result<(), DomainError> {
            Ok(())
        }

        async fn unstage_lines(&self, _repo: &Path, _file: &Path, _idx: usize, _lines: &[usize]) -> Result<(), DomainError> {
            Ok(())
        }

        async fn discard_changes(&self, _repo: &Path, _file: &Path) -> Result<(), DomainError> {
            Ok(())
        }

        async fn create_commit(&self, _repo: &Path, _msg: &str, _author: &GitAuthor) -> Result<String, DomainError> {
            Ok("abc1234".into())
        }

        async fn write_resolved_conflict(&self, _repo: &Path, _file: &Path, _content: &str) -> Result<(), DomainError> {
            Ok(())
        }

        async fn clone_repository(&self, _url: &str, _destination: &Path) -> Result<(), DomainError> {
            Ok(())
        }

        async fn stage_all(&self, _repo: &Path) -> Result<(), DomainError> {
            Ok(())
        }

        async fn unstage_all(&self, _repo: &Path) -> Result<(), DomainError> {
            Ok(())
        }

        async fn get_tags(&self, _repo: &Path) -> Result<Vec<rmerge_domain::entities::Tag>, DomainError> {
            Ok(vec![])
        }

        async fn create_tag(&self, _repo: &Path, _name: &str, _target_commit: Option<&str>, _message: Option<&str>) -> Result<(), DomainError> {
            Ok(())
        }

        async fn delete_tag(&self, _repo: &Path, _name: &str) -> Result<(), DomainError> {
            Ok(())
        }

        async fn create_branch(&self, _repo: &Path, _name: &str, _start_point: Option<&str>, _checkout: bool) -> Result<(), DomainError> {
            Ok(())
        }

        async fn switch_branch(&self, _repo: &Path, _name: &str) -> Result<(), DomainError> {
            Ok(())
        }

        async fn merge_branch(&self, _repo: &Path, _source_branch: &str) -> Result<rmerge_domain::entities::MergeOutcome, DomainError> {
            Ok(rmerge_domain::entities::MergeOutcome::UpToDate)
        }

        async fn get_commit_detail(&self, _repo: &Path, commit_id: &str) -> Result<rmerge_domain::entities::CommitDetail, DomainError> {
            Ok(rmerge_domain::entities::CommitDetail {
                commit: Commit {
                    id: commit_id.to_string(),
                    short_id: commit_id[..7.min(commit_id.len())].to_string(),
                    message_headline: "Headline".into(),
                    message_body: None,
                    author_name: "Author".into(),
                    author_email: "author@test.com".into(),
                    authored_at: chrono::Utc::now(),
                    parent_ids: vec![],
                    lane: 0,
                    branches: vec![],
                    tags: vec![],
                },
                tree_id: "tree123".into(),
                stats: rmerge_domain::entities::CommitStats::default(),
                files: vec![],
            })
        }

        async fn get_remote_url(&self, _repo: &Path, _remote_name: Option<&str>) -> Result<Option<String>, DomainError> {
            Ok(None)
        }

        async fn get_gitignore(&self, _repo: &Path) -> Result<Option<String>, DomainError> {
            Ok(None)
        }

        async fn save_gitignore(&self, _repo: &Path, _content: &str) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_commit_requires_author_when_none_configured() {
        let mock_cfg = Arc::new(MockGitConfig {
            author: Mutex::new(None),
            last_scope: Mutex::new(None),
        });
        let mock_storage = Arc::new(MockGitStorage { staged: true });
        let commit_service = CommitService::new(mock_storage, mock_cfg);

        let result = commit_service.execute(Path::new("/dummy"), "Initial commit").await.unwrap();
        assert_eq!(result, CommitOutcome::AuthorRequired);
    }

    #[tokio::test]
    async fn test_commit_succeeds_when_author_configured() {
        let mock_cfg = Arc::new(MockGitConfig {
            author: Mutex::new(Some(GitAuthor::new("Anibal", "anibal@example.com"))),
            last_scope: Mutex::new(None),
        });
        let mock_storage = Arc::new(MockGitStorage { staged: true });
        let commit_service = CommitService::new(mock_storage, mock_cfg);

        let result = commit_service.execute(Path::new("/dummy"), "Initial commit").await.unwrap();
        assert_eq!(result, CommitOutcome::Success { commit_hash: "abc1234".into() });
    }

    #[tokio::test]
    async fn test_configure_author_saves_with_correct_scope() {
        let mock_cfg = Arc::new(MockGitConfig {
            author: Mutex::new(None),
            last_scope: Mutex::new(None),
        });
        let author_service = AuthorService::new(mock_cfg.clone());

        let author = GitAuthor::new("Anibal GH", "anibal@example.com");
        author_service.configure_author(Path::new("/dummy"), author.clone(), ConfigScope::Global).await.unwrap();

        assert_eq!(*mock_cfg.last_scope.lock().unwrap(), Some(ConfigScope::Global));
        assert_eq!(*mock_cfg.author.lock().unwrap(), Some(author));
    }

    #[tokio::test]
    async fn test_clone_repository_delegates_to_storage() {
        let mock_storage = Arc::new(MockGitStorage { staged: false });
        let repo_service = RepositoryService::new(mock_storage);

        let result = repo_service.clone_repo("https://github.com/test/repo.git", Path::new("/tmp/test-repo")).await;
        assert!(result.is_ok());
    }
}
