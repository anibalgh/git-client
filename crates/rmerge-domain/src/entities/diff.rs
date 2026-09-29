use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineKind {
    Context,
    Addition,
    Deletion,
    Header,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub content: String,
    pub kind: LineKind,
    pub is_selected: bool,
}

impl DiffLine {
    pub fn context(old: u32, new: u32, content: impl Into<String>) -> Self {
        Self {
            old_line_no: Some(old),
            new_line_no: Some(new),
            content: content.into(),
            kind: LineKind::Context,
            is_selected: false,
        }
    }

    pub fn addition(new: u32, content: impl Into<String>) -> Self {
        Self {
            old_line_no: None,
            new_line_no: Some(new),
            content: content.into(),
            kind: LineKind::Addition,
            is_selected: false,
        }
    }

    pub fn deletion(old: u32, content: impl Into<String>) -> Self {
        Self {
            old_line_no: Some(old),
            new_line_no: None,
            content: content.into(),
            kind: LineKind::Deletion,
            is_selected: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hunk {
    pub id: usize,
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: String,
    pub old_path: Option<String>,
    pub status: DeltaKind,
    pub is_staged: bool,
    pub hunks: Vec<Hunk>,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingArea {
    pub staged: Vec<FilePatch>,
    pub unstaged: Vec<FilePatch>,
    pub untracked: Vec<String>,
    pub conflicts: Vec<String>,
}
