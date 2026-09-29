use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IconId {
    GitCommit,
    GitBranch,
    GitMerge,
    GitPullRequest,
    GitCompare,
    DiffAdded,
    DiffRemoved,
    DiffModified,
    Stage,
    Unstage,
    Discard,
    Tag,
    Search,
    Settings,
    User,
    Check,
    AlertTriangle,
    FolderGit,
    Terminal,
}
