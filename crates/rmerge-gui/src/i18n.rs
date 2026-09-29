use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translations {
    pub app_name: String,
    pub open_repo: String,
    pub open_repo_dialog_title: String,
    pub refresh: String,
    pub home: String,
    pub show_graph: String,
    pub hide_graph: String,
    pub toggle_graph_tooltip: String,
    pub theme: String,
    pub font_size: String,
    pub language: String,

    pub git_explorer: String,
    pub branch_menu: String,
    pub menu_edit_gitignore: String,
    pub menu_new_branch: String,
    pub menu_switch_branch: String,
    pub menu_merge: String,
    pub menu_new_tag: String,
    pub menu_create_pr: String,
    pub menu_refresh: String,
    pub refresh_tooltip: String,

    pub branches_header: String,
    pub branch_list: String,
    pub new_branch_tooltip: String,
    pub switch_branch_btn: String,
    pub switch_branch_tooltip: String,
    pub push_actions_tooltip: String,
    pub pull_actions_tooltip: String,

    pub tags_header: String,
    pub tag_list: String,
    pub new_tag_btn: String,
    pub new_tag_tooltip: String,
    pub no_tags: String,
    pub delete_tag_tooltip: String,

    pub conflicts_header: String,
    pub resolve_conflict: String,

    pub staged_header: String,
    pub staged_view: String,
    pub unstage_all_tooltip: String,
    pub unstage_file_tooltip: String,
    pub no_staged_changes: String,

    pub unstaged_header: String,
    pub unstaged_view: String,
    pub stage_all_tooltip: String,
    pub stage_file_tooltip: String,
    pub no_unstaged_changes: String,

    pub untracked_header: String,
    pub stage_untracked_tooltip: String,

    pub commit_message_label: String,
    pub commit_message_hint: String,
    pub commit_button: String,

    pub commit_history_title: String,
    pub commits_count: String,
    pub graph_col: String,
    pub hash_col: String,
    pub message_col: String,
    pub author_col: String,
    pub date_col: String,

    pub commit_detail_title: String,
    pub commit_label: String,
    pub tree_label: String,
    pub author_label: String,
    pub date_label: String,
    pub parents_label: String,
    pub branches_label: String,
    pub root_commit: String,
    pub copy_hash: String,
    pub copy_hash_tooltip: String,
    pub files_changed_stat: String,
    pub insertions_stat: String,
    pub deletions_stat: String,

    pub modified_files_label: String,
    pub select_file_placeholder: String,
    pub diff_status_added: String,
    pub diff_status_deleted: String,
    pub diff_status_renamed: String,
    pub diff_status_untracked: String,
    pub diff_status_conflicted: String,
    pub diff_status_modified: String,
    pub diff_status_badge: String,

    pub branch_graph_title: String,
    pub branch_graph_close: String,
    pub branch_graph_selector: String,
    pub all_branches_option: String,
    pub graph_legend_title: String,
    pub graph_legend_standard: String,
    pub graph_legend_merge: String,
    pub graph_legend_fork: String,
    pub graph_legend_head: String,

    pub welcome_title: String,
    pub welcome_subtitle: String,
    pub recent_repos_title: String,
    pub open_local_repo: String,
    pub clone_repo: String,
    pub clear_history: String,
    pub no_recent_repos: String,

    pub modal_create_branch_title: String,
    pub modal_branch_name_label: String,
    pub modal_branch_start_label: String,
    pub modal_create_btn: String,
    pub modal_cancel_btn: String,

    pub modal_switch_branch_title: String,
    pub modal_switch_target_label: String,
    pub modal_switch_btn: String,

    pub modal_merge_title: String,
    pub modal_merge_target_label: String,
    pub modal_merge_btn: String,

    pub modal_tag_title: String,
    pub modal_tag_name_label: String,
    pub modal_tag_message_label: String,
    pub modal_tag_commit_label: String,
    pub modal_create_tag_btn: String,

    pub modal_gitignore_title: String,
    pub modal_save_gitignore_btn: String,

    pub preferences_title: String,
    pub themes_title: String,
    pub themes_subtitle: String,
    pub typography_title: String,
    pub ui_font_section: String,
    pub code_font_section: String,
    pub font_family_label: String,
    pub ui_font_size_label: String,
    pub code_font_size_label: String,
    pub enable_ligatures_label: String,
    pub close_btn: String,

    pub about_title: String,
    pub about_btn: String,
    pub about_version_label: String,
    pub about_description: String,
    pub about_license_label: String,
    pub about_license_value: String,
    pub about_author_label: String,
    pub about_platform_label: String,
    pub about_platform_value: String,
    pub about_architecture_label: String,
    pub about_architecture_value: String,

    pub server_unreachable_title: String,
    pub server_unreachable_timeout: String,
    pub server_unreachable_refused: String,
    pub server_unreachable_hint: String,
    pub server_unreachable_cmd_failed: String,
    pub server_unreachable_action_btn: String,
    pub running_command: String,
}

impl Translations {
    pub fn load(lang: &str) -> Self {
        // Primero intenta cargar de archivos locales si existen en runtime
        let local_path = format!("locales/{lang}.json");
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            if let Ok(trans) = serde_json::from_str(&content) {
                return trans;
            }
        }

        let json_str = match lang {
            "en" => include_str!("../../../locales/en.json"),
            _ => include_str!("../../../locales/es.json"),
        };

        serde_json::from_str(json_str).unwrap_or_else(|_| {
            serde_json::from_str(include_str!("../../../locales/es.json"))
                .expect("Valid embedded Spanish translation")
        })
    }
}
