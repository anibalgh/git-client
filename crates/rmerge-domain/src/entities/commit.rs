use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub message_headline: String,
    pub message_body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub authored_at: DateTime<Utc>,
    pub parent_ids: Vec<String>,
    pub lane: usize,
    pub branches: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub target_commit_id: String,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub target_commit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitGraphSegment {
    pub from_lane: usize,
    pub to_lane: usize,
    pub target_commit_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommitGraph {
    pub commits: Vec<Commit>,
    pub max_lanes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CommitStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitFileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub status: super::diff::DeltaKind,
    pub additions: usize,
    pub deletions: usize,
    pub patch: super::diff::FilePatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitDetail {
    pub commit: Commit,
    pub tree_id: String,
    pub stats: CommitStats,
    pub files: Vec<CommitFileDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeOutcome {
    FastForward { new_commit_id: String },
    Merged { merge_commit_id: String },
    Conflicts { conflict_files: Vec<String> },
    UpToDate,
}
