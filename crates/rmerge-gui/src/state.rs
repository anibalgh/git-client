use std::path::PathBuf;
use rmerge_application::ports::r#in::{AvailableFonts, RepositoryData};
use rmerge_domain::entities::{CommitDetail, ConfigScope, ThreeWayMergeFile};
use rmerge_domain::value_objects::{ThemeConfig, TypographyConfig};
use crate::i18n::Translations;

#[derive(Debug, Clone)]
pub struct IdentityModalState {
    pub name: String,
    pub email: String,
    pub scope: ConfigScope,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub repo_path: PathBuf,
    pub repo_data: Option<RepositoryData>,
    pub selected_file: Option<String>,
    pub selected_is_staged: bool,
    pub commit_message: String,
    pub active_theme: ThemeConfig,
    pub available_themes: Vec<String>,
    pub typography: TypographyConfig,
    pub available_fonts: AvailableFonts,
    pub identity_modal: Option<IdentityModalState>,
    pub show_preferences: bool,
    pub active_merge: Option<ThreeWayMergeFile>,
    pub status_message: Option<String>,
    pub is_valid_repo: bool,
    pub recent_repos: Vec<String>,
    pub show_clone_modal: bool,
    pub clone_url: String,
    pub clone_destination: String,
    pub clone_error: Option<String>,
    pub is_cloning: bool,
    pub show_manual_open_modal: bool,
    pub manual_open_path: String,

    pub language: String,
    pub i18n: Translations,

    // Detalle de commit seleccionado
    pub selected_commit_id: Option<String>,
    pub selected_commit_detail: Option<CommitDetail>,
    pub selected_commit_file: Option<String>,

    // Panel lateral derecho para grafo de cambios de la rama seleccionada
    pub show_right_graph_panel: bool,
    pub selected_branch_for_graph: Option<String>,

    // Modal para crear Tag
    pub show_create_tag_modal: bool,
    pub new_tag_name: String,
    pub new_tag_target: String,
    pub new_tag_message: String,
    pub create_tag_error: Option<String>,

    // Modal para crear Rama
    pub show_create_branch_modal: bool,
    pub new_branch_name: String,
    pub new_branch_start: String,
    pub new_branch_checkout: bool,
    pub create_branch_error: Option<String>,

    // Modal para cambiar de Rama (Switch / Checkout)
    pub show_switch_branch_modal: bool,
    pub switch_branch_target: String,
    pub switch_branch_error: Option<String>,

    // Modal para Merge
    pub show_merge_modal: bool,
    pub merge_source_branch: String,
    pub merge_error: Option<String>,

    // Modal para editar / crear .gitignore
    pub show_gitignore_modal: bool,
    pub gitignore_content: String,
    pub gitignore_exists_on_disk: bool,
    pub gitignore_save_error: Option<String>,

    pub fonts_initialized: bool,
    pub show_about_modal: bool,
    pub is_running_git_cmd: bool,
    pub active_git_cmd_name: Option<String>,
    pub server_unreachable_modal: Option<UnreachableModalData>,
}

#[derive(Debug, Clone)]
pub struct UnreachableModalData {
    pub cmd: String,
    pub host: String,
    pub port: u16,
    pub is_timeout: bool,
    pub error_detail: Option<String>,
}

impl AppState {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            repo_data: None,
            selected_file: None,
            selected_is_staged: false,
            commit_message: String::new(),
            active_theme: ThemeConfig::dark(),
            available_themes: vec![
                "Sublime Dark".into(),
                "Sublime Light".into(),
                "Monokai Pro".into(),
                "Kanagawa Wave".into(),
                "Kanagawa Dragon".into(),
                "Kanagawa Lotus".into(),
                "Gruvbox Dark".into(),
                "Gruvbox Light".into(),
                "Dracula".into(),
                "Nord".into(),
                "Tokyo Night".into(),
                "Catppuccin Mocha".into(),
                "Catppuccin Latte".into(),
                "One Dark Pro".into(),
                "Rose Pine".into(),
                "Solarized Dark".into(),
                "Solarized Light".into(),
            ],
            typography: TypographyConfig::default(),
            available_fonts: AvailableFonts {
                system_fonts: vec!["Inter".into(), "Segoe UI".into(), "SF Pro Text".into()],
                monospace_fonts: vec!["JetBrains Mono".into(), "Fira Code".into(), "Cascadia Code".into(), "monospace".into()],
            },
            identity_modal: None,
            show_preferences: false,
            active_merge: None,
            status_message: None,
            is_valid_repo: false,
            recent_repos: Vec::new(),
            show_clone_modal: false,
            clone_url: String::new(),
            clone_destination: String::new(),
            clone_error: None,
            is_cloning: false,
            show_manual_open_modal: false,
            manual_open_path: String::new(),

            fonts_initialized: false,
            show_about_modal: false,
            is_running_git_cmd: false,
            active_git_cmd_name: None,
            server_unreachable_modal: None,

            language: "es".to_string(),
            i18n: Translations::load("es"),

            selected_commit_id: None,
            selected_commit_detail: None,
            selected_commit_file: None,

            show_right_graph_panel: true,
            selected_branch_for_graph: None,

            show_create_tag_modal: false,
            new_tag_name: String::new(),
            new_tag_target: String::new(),
            new_tag_message: String::new(),
            create_tag_error: None,

            show_create_branch_modal: false,
            new_branch_name: String::new(),
            new_branch_start: String::new(),
            new_branch_checkout: true,
            create_branch_error: None,

            show_switch_branch_modal: false,
            switch_branch_target: String::new(),
            switch_branch_error: None,

            show_merge_modal: false,
            merge_source_branch: String::new(),
            merge_error: None,

            show_gitignore_modal: false,
            gitignore_content: String::new(),
            gitignore_exists_on_disk: false,
            gitignore_save_error: None,
        }
    }

    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        self.i18n = Translations::load(lang);
    }

    pub fn add_recent_repo(&mut self, path: String) {
        let trimmed = path.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        self.recent_repos.retain(|p| p != &trimmed);
        self.recent_repos.insert(0, trimmed);
        if self.recent_repos.len() > 30 {
            self.recent_repos.truncate(30);
        }
    }

    pub fn remove_recent_repo(&mut self, path: &str) {
        self.recent_repos.retain(|p| p != path);
    }

    pub fn clear_recent_repos(&mut self) {
        self.recent_repos.clear();
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn open_identity_modal(&mut self, suggested_name: Option<String>, suggested_email: Option<String>) {
        self.identity_modal = Some(IdentityModalState {
            name: suggested_name.unwrap_or_default(),
            email: suggested_email.unwrap_or_default(),
            scope: ConfigScope::Local,
            error_message: None,
        });
    }

    #[allow(dead_code)]
    pub fn close_identity_modal(&mut self) {
        self.identity_modal = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_repos_ordering_and_deduplication() {
        let mut state = AppState::new(PathBuf::from("."));
        assert!(state.recent_repos.is_empty());

        state.add_recent_repo("/repo/first".into());
        state.add_recent_repo("/repo/second".into());
        state.add_recent_repo("/repo/third".into());

        // Debe estar ordenado de más reciente a más antiguo
        assert_eq!(state.recent_repos, vec![
            "/repo/third",
            "/repo/second",
            "/repo/first",
        ]);

        // Si se vuelve a abrir un repo existente, debe moverse al frente (índice 0)
        state.add_recent_repo("/repo/first".into());
        assert_eq!(state.recent_repos, vec![
            "/repo/first",
            "/repo/third",
            "/repo/second",
        ]);
    }

    #[test]
    fn test_remove_and_clear_recent_repos() {
        let mut state = AppState::new(PathBuf::from("."));
        state.add_recent_repo("/repo/one".into());
        state.add_recent_repo("/repo/two".into());
        state.add_recent_repo("/repo/three".into());

        state.remove_recent_repo("/repo/two");
        assert_eq!(state.recent_repos, vec!["/repo/three", "/repo/one"]);

        state.clear_recent_repos();
        assert!(state.recent_repos.is_empty());
    }
}
