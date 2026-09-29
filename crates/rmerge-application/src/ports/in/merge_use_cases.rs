use rmerge_domain::entities::{ConflictResolution, ThreeWayMergeFile};
use rmerge_domain::errors::DomainError;
use std::path::Path;

#[async_trait::async_trait]
pub trait ThreeWayMergeUseCase: Send + Sync {
    async fn load_conflicted_file(&self, repo_path: &Path, file_path: &Path) -> Result<ThreeWayMergeFile, DomainError>;
    async fn apply_hunk_resolution(
        &self,
        merge_file: &mut ThreeWayMergeFile,
        block_id: usize,
        resolution: ConflictResolution,
    ) -> Result<(), DomainError>;
    async fn save_and_resolve(&self, repo_path: &Path, merge_file: &ThreeWayMergeFile) -> Result<(), DomainError>;
}
