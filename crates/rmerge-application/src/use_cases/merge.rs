use std::path::Path;
use std::sync::Arc;
use rmerge_domain::entities::{ConflictResolution, ThreeWayMergeFile};
use rmerge_domain::errors::DomainError;
use rmerge_domain::services::ThreeWayMergeService;
use crate::ports::r#in::ThreeWayMergeUseCase;
use crate::ports::out::GitStoragePort;

pub struct MergeService {
    git_storage: Arc<dyn GitStoragePort>,
}

impl MergeService {
    pub fn new(git_storage: Arc<dyn GitStoragePort>) -> Self {
        Self { git_storage }
    }
}

#[async_trait::async_trait]
impl ThreeWayMergeUseCase for MergeService {
    async fn load_conflicted_file(&self, repo_path: &Path, file_path: &Path) -> Result<ThreeWayMergeFile, DomainError> {
        let full_path = repo_path.join(file_path);
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| DomainError::Io(e.to_string()))?;

        let merge_file = ThreeWayMergeService::parse_conflicted_content(
            file_path.to_string_lossy().as_ref(),
            &content,
        );

        Ok(merge_file)
    }

    async fn apply_hunk_resolution(
        &self,
        merge_file: &mut ThreeWayMergeFile,
        block_id: usize,
        resolution: ConflictResolution,
    ) -> Result<(), DomainError> {
        let block = merge_file
            .blocks
            .iter_mut()
            .find(|b| b.id == block_id)
            .ok_or(DomainError::HunkNotFound)?;

        block.apply_resolution(resolution);
        Ok(())
    }

    async fn save_and_resolve(&self, repo_path: &Path, merge_file: &ThreeWayMergeFile) -> Result<(), DomainError> {
        let assembled = merge_file
            .assemble_merged_file()
            .ok_or_else(|| DomainError::UnresolvedConflict("Existen bloques de conflicto sin resolver.".into()))?;

        let file_path = Path::new(&merge_file.file_path);
        self.git_storage.write_resolved_conflict(repo_path, file_path, &assembled).await
    }
}
