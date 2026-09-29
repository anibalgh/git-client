use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseOurs,
    UseTheirs,
    UseBase,
    UseBoth { ours_first: bool },
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictBlock {
    pub id: usize,
    pub ancestor_content: String,
    pub ours_content: String,
    pub theirs_content: String,
    pub active_resolution: Option<ConflictResolution>,
    pub resolved_content: Option<String>,
}

impl ConflictBlock {
    pub fn new(
        id: usize,
        ancestor: impl Into<String>,
        ours: impl Into<String>,
        theirs: impl Into<String>,
    ) -> Self {
        Self {
            id,
            ancestor_content: ancestor.into(),
            ours_content: ours.into(),
            theirs_content: theirs.into(),
            active_resolution: None,
            resolved_content: None,
        }
    }

    pub fn apply_resolution(&mut self, resolution: ConflictResolution) {
        let content = match &resolution {
            ConflictResolution::UseOurs => self.ours_content.clone(),
            ConflictResolution::UseTheirs => self.theirs_content.clone(),
            ConflictResolution::UseBase => self.ancestor_content.clone(),
            ConflictResolution::UseBoth { ours_first: true } => {
                format!("{}\n{}", self.ours_content.trim_end(), self.theirs_content.trim_start())
            }
            ConflictResolution::UseBoth { ours_first: false } => {
                format!("{}\n{}", self.theirs_content.trim_end(), self.ours_content.trim_start())
            }
            ConflictResolution::Custom(text) => text.clone(),
        };
        self.resolved_content = Some(content);
        self.active_resolution = Some(resolution);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreeWayMergeFile {
    pub file_path: String,
    pub blocks: Vec<ConflictBlock>,
}

impl ThreeWayMergeFile {
    pub fn is_fully_resolved(&self) -> bool {
        !self.blocks.is_empty() && self.blocks.iter().all(|b| b.resolved_content.is_some())
    }

    pub fn assemble_merged_file(&self) -> Option<String> {
        if !self.is_fully_resolved() {
            return None;
        }
        let merged = self.blocks
            .iter()
            .filter_map(|b| b.resolved_content.as_deref())
            .collect::<Vec<_>>()
            .join("\n");
        Some(merged)
    }
}
