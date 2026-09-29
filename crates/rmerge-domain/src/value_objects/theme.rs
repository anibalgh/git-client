use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub mode: ThemeMode,
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_surface: String,
    pub fg_primary: String,
    pub fg_muted: String,
    pub border_color: String,
    pub accent: String,
    // Diffs
    pub diff_add_bg: String,
    pub diff_add_fg: String,
    pub diff_del_bg: String,
    pub diff_del_fg: String,
    pub diff_mod_bg: String,
    pub diff_mod_fg: String,
    // Conflictos
    pub conflict_bg: String,
    pub conflict_border: String,
    // Carriles del grafo de commits
    pub branch_lanes: Vec<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeConfig {
    pub fn dark() -> Self {
        Self {
            name: "Sublime Dark".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#181a1f".into(),
            bg_secondary: "#21252b".into(),
            bg_surface: "#282c34".into(),
            fg_primary: "#abb2bf".into(),
            fg_muted: "#5c6370".into(),
            border_color: "#181a1f".into(),
            accent: "#61afef".into(),
            diff_add_bg: "#213627".into(),
            diff_add_fg: "#98c379".into(),
            diff_del_bg: "#3b1e22".into(),
            diff_del_fg: "#e06c75".into(),
            diff_mod_bg: "#323023".into(),
            diff_mod_fg: "#e5c07b".into(),
            conflict_bg: "#47381b".into(),
            conflict_border: "#d19a66".into(),
            branch_lanes: vec![
                "#e06c75".into(),
                "#61afef".into(),
                "#98c379".into(),
                "#d19a66".into(),
                "#c678dd".into(),
                "#56b6c2".into(),
            ],
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Sublime Light".into(),
            mode: ThemeMode::Light,
            bg_primary: "#fafafa".into(),
            bg_secondary: "#f0f0f0".into(),
            bg_surface: "#ffffff".into(),
            fg_primary: "#383a42".into(),
            fg_muted: "#a0a1a7".into(),
            border_color: "#e5e5e6".into(),
            accent: "#4078f2".into(),
            diff_add_bg: "#e6ffec".into(),
            diff_add_fg: "#22863a".into(),
            diff_del_bg: "#ffeef0".into(),
            diff_del_fg: "#cb2431".into(),
            diff_mod_bg: "#fff5b1".into(),
            diff_mod_fg: "#b08800".into(),
            conflict_bg: "#fffbdd".into(),
            conflict_border: "#d73a49".into(),
            branch_lanes: vec![
                "#e45649".into(),
                "#4078f2".into(),
                "#50a14f".into(),
                "#986801".into(),
                "#a626a4".into(),
                "#0184bc".into(),
            ],
        }
    }
}
