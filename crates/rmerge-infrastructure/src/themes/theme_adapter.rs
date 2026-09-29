use std::collections::HashMap;
use rmerge_application::ports::out::ThemeStoragePort;
use rmerge_domain::errors::DomainError;
use rmerge_domain::value_objects::{ThemeConfig, ThemeMode};

pub struct EmbeddedThemeAdapter {
    themes: HashMap<String, ThemeConfig>,
}

impl EmbeddedThemeAdapter {
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        let list = vec![
            Self::sublime_dark(),
            Self::sublime_light(),
            Self::monokai_pro(),
            Self::kanagawa_wave(),
            Self::kanagawa_dragon(),
            Self::kanagawa_lotus(),
            Self::gruvbox_dark(),
            Self::gruvbox_light(),
            Self::dracula(),
            Self::nord(),
            Self::tokyo_night(),
            Self::catppuccin_mocha(),
            Self::catppuccin_latte(),
            Self::one_dark_pro(),
            Self::rose_pine(),
            Self::solarized_dark(),
            Self::solarized_light(),
        ];

        for theme in list {
            themes.insert(theme.name.clone(), theme);
        }

        Self { themes }
    }

    pub fn sublime_dark() -> ThemeConfig {
        ThemeConfig::dark()
    }

    pub fn sublime_light() -> ThemeConfig {
        ThemeConfig::light()
    }

    pub fn monokai_pro() -> ThemeConfig {
        ThemeConfig {
            name: "Monokai Pro".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#2d2a2e".into(),
            bg_secondary: "#221f22".into(),
            bg_surface: "#403e41".into(),
            fg_primary: "#fcfcfa".into(),
            fg_muted: "#727072".into(),
            border_color: "#19181a".into(),
            accent: "#ffd866".into(),
            diff_add_bg: "#2b3b2b".into(),
            diff_add_fg: "#a9dc76".into(),
            diff_del_bg: "#42242b".into(),
            diff_del_fg: "#ff6188".into(),
            diff_mod_bg: "#3b3726".into(),
            diff_mod_fg: "#ffd866".into(),
            conflict_bg: "#4d3822".into(),
            conflict_border: "#fc9867".into(),
            branch_lanes: vec![
                "#ff6188".into(),
                "#fc9867".into(),
                "#ffd866".into(),
                "#a9dc76".into(),
                "#78dce8".into(),
                "#ab9df2".into(),
            ],
        }
    }

    pub fn kanagawa_wave() -> ThemeConfig {
        ThemeConfig {
            name: "Kanagawa Wave".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#1f1f28".into(),
            bg_secondary: "#16161d".into(),
            bg_surface: "#2a2a37".into(),
            fg_primary: "#dcd7ba".into(),
            fg_muted: "#727169".into(),
            border_color: "#16161d".into(),
            accent: "#7e9cd8".into(),
            diff_add_bg: "#233327".into(),
            diff_add_fg: "#76946a".into(),
            diff_del_bg: "#43242b".into(),
            diff_del_fg: "#c34043".into(),
            diff_mod_bg: "#2d4f67".into(),
            diff_mod_fg: "#e6c384".into(),
            conflict_bg: "#3b2c1f".into(),
            conflict_border: "#ffa066".into(),
            branch_lanes: vec![
                "#7e9cd8".into(),
                "#98bb6c".into(),
                "#ffa066".into(),
                "#e46876".into(),
                "#957fb8".into(),
                "#7aa89f".into(),
            ],
        }
    }

    pub fn kanagawa_dragon() -> ThemeConfig {
        ThemeConfig {
            name: "Kanagawa Dragon".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#181616".into(),
            bg_secondary: "#12120f".into(),
            bg_surface: "#282727".into(),
            fg_primary: "#c5c9c5".into(),
            fg_muted: "#625e5a".into(),
            border_color: "#12120f".into(),
            accent: "#8ba4b0".into(),
            diff_add_bg: "#243324".into(),
            diff_add_fg: "#87a987".into(),
            diff_del_bg: "#3f2222".into(),
            diff_del_fg: "#c4746e".into(),
            diff_mod_bg: "#323023".into(),
            diff_mod_fg: "#c4b28a".into(),
            conflict_bg: "#3e2b1f".into(),
            conflict_border: "#b6927b".into(),
            branch_lanes: vec![
                "#8ba4b0".into(),
                "#87a987".into(),
                "#c4746e".into(),
                "#c4b28a".into(),
                "#a292a3".into(),
                "#8a9a86".into(),
            ],
        }
    }

    pub fn kanagawa_lotus() -> ThemeConfig {
        ThemeConfig {
            name: "Kanagawa Lotus".into(),
            mode: ThemeMode::Light,
            bg_primary: "#f2ecbc".into(),
            bg_secondary: "#e5ddb0".into(),
            bg_surface: "#ded5a5".into(),
            fg_primary: "#545464".into(),
            fg_muted: "#8a8980".into(),
            border_color: "#d5cea3".into(),
            accent: "#4d699b".into(),
            diff_add_bg: "#e1ecc8".into(),
            diff_add_fg: "#436440".into(),
            diff_del_bg: "#f5d5d8".into(),
            diff_del_fg: "#c84053".into(),
            diff_mod_bg: "#fce8b3".into(),
            diff_mod_fg: "#77713f".into(),
            conflict_bg: "#f9e7c4".into(),
            conflict_border: "#b85934".into(),
            branch_lanes: vec![
                "#4d699b".into(),
                "#436440".into(),
                "#b85934".into(),
                "#c84053".into(),
                "#624c7c".into(),
                "#4e8ca2".into(),
            ],
        }
    }

    pub fn gruvbox_dark() -> ThemeConfig {
        ThemeConfig {
            name: "Gruvbox Dark".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#282828".into(),
            bg_secondary: "#1d2021".into(),
            bg_surface: "#3c3836".into(),
            fg_primary: "#ebdbb2".into(),
            fg_muted: "#928374".into(),
            border_color: "#1d2021".into(),
            accent: "#fe8019".into(),
            diff_add_bg: "#32361a".into(),
            diff_add_fg: "#b8bb26".into(),
            diff_del_bg: "#3c1f1e".into(),
            diff_del_fg: "#fb4934".into(),
            diff_mod_bg: "#3b381d".into(),
            diff_mod_fg: "#fabd2f".into(),
            conflict_bg: "#422e1b".into(),
            conflict_border: "#fe8019".into(),
            branch_lanes: vec![
                "#fb4934".into(),
                "#fe8019".into(),
                "#fabd2f".into(),
                "#b8bb26".into(),
                "#83a598".into(),
                "#d3869b".into(),
            ],
        }
    }

    pub fn gruvbox_light() -> ThemeConfig {
        ThemeConfig {
            name: "Gruvbox Light".into(),
            mode: ThemeMode::Light,
            bg_primary: "#fbf1c7".into(),
            bg_secondary: "#f2e5bc".into(),
            bg_surface: "#ebdbb2".into(),
            fg_primary: "#3c3836".into(),
            fg_muted: "#928374".into(),
            border_color: "#d5c4a1".into(),
            accent: "#af3a03".into(),
            diff_add_bg: "#e2e8c0".into(),
            diff_add_fg: "#79740e".into(),
            diff_del_bg: "#f2d5cf".into(),
            diff_del_fg: "#9d0006".into(),
            diff_mod_bg: "#f5eab0".into(),
            diff_mod_fg: "#b57614".into(),
            conflict_bg: "#f6ded2".into(),
            conflict_border: "#af3a03".into(),
            branch_lanes: vec![
                "#9d0006".into(),
                "#af3a03".into(),
                "#b57614".into(),
                "#79740e".into(),
                "#076678".into(),
                "#8f3f71".into(),
            ],
        }
    }

    pub fn dracula() -> ThemeConfig {
        ThemeConfig {
            name: "Dracula".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#282a36".into(),
            bg_secondary: "#21222c".into(),
            bg_surface: "#44475a".into(),
            fg_primary: "#f8f8f2".into(),
            fg_muted: "#6272a4".into(),
            border_color: "#191a21".into(),
            accent: "#bd93f9".into(),
            diff_add_bg: "#23382c".into(),
            diff_add_fg: "#50fa7b".into(),
            diff_del_bg: "#3d1f28".into(),
            diff_del_fg: "#ff5555".into(),
            diff_mod_bg: "#3e3825".into(),
            diff_mod_fg: "#f1fa8c".into(),
            conflict_bg: "#422f20".into(),
            conflict_border: "#ffb86c".into(),
            branch_lanes: vec![
                "#bd93f9".into(),
                "#ff79c6".into(),
                "#8be9fd".into(),
                "#50fa7b".into(),
                "#ffb86c".into(),
                "#f1fa8c".into(),
            ],
        }
    }

    pub fn nord() -> ThemeConfig {
        ThemeConfig {
            name: "Nord".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#2e3440".into(),
            bg_secondary: "#242933".into(),
            bg_surface: "#3b4252".into(),
            fg_primary: "#eceff4".into(),
            fg_muted: "#616e88".into(),
            border_color: "#242933".into(),
            accent: "#88c0d0".into(),
            diff_add_bg: "#283b38".into(),
            diff_add_fg: "#a3be8c".into(),
            diff_del_bg: "#3e282c".into(),
            diff_del_fg: "#bf616a".into(),
            diff_mod_bg: "#3e382b".into(),
            diff_mod_fg: "#ebcb8b".into(),
            conflict_bg: "#41322b".into(),
            conflict_border: "#d08770".into(),
            branch_lanes: vec![
                "#88c0d0".into(),
                "#81a1c1".into(),
                "#5e81ac".into(),
                "#a3be8c".into(),
                "#ebcb8b".into(),
                "#b48ead".into(),
            ],
        }
    }

    pub fn tokyo_night() -> ThemeConfig {
        ThemeConfig {
            name: "Tokyo Night".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#1a1b26".into(),
            bg_secondary: "#16161e".into(),
            bg_surface: "#24283b".into(),
            fg_primary: "#c0caf5".into(),
            fg_muted: "#565f89".into(),
            border_color: "#16161e".into(),
            accent: "#7aa2f7".into(),
            diff_add_bg: "#1c352d".into(),
            diff_add_fg: "#9ece6a".into(),
            diff_del_bg: "#3c202a".into(),
            diff_del_fg: "#f7768e".into(),
            diff_mod_bg: "#393424".into(),
            diff_mod_fg: "#e0af68".into(),
            conflict_bg: "#3b2c20".into(),
            conflict_border: "#ff9e64".into(),
            branch_lanes: vec![
                "#7aa2f7".into(),
                "#bb9af7".into(),
                "#7dcfff".into(),
                "#9ece6a".into(),
                "#ff9e64".into(),
                "#f7768e".into(),
            ],
        }
    }

    pub fn catppuccin_mocha() -> ThemeConfig {
        ThemeConfig {
            name: "Catppuccin Mocha".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#1e1e2e".into(),
            bg_secondary: "#181825".into(),
            bg_surface: "#313244".into(),
            fg_primary: "#cdd6f4".into(),
            fg_muted: "#6c7086".into(),
            border_color: "#11111b".into(),
            accent: "#89b4fa".into(),
            diff_add_bg: "#233830".into(),
            diff_add_fg: "#a6e3a1".into(),
            diff_del_bg: "#3b202c".into(),
            diff_del_fg: "#f38ba8".into(),
            diff_mod_bg: "#383428".into(),
            diff_mod_fg: "#f9e2af".into(),
            conflict_bg: "#3a2b22".into(),
            conflict_border: "#fab387".into(),
            branch_lanes: vec![
                "#89b4fa".into(),
                "#cba6f7".into(),
                "#f38ba8".into(),
                "#fab387".into(),
                "#a6e3a1".into(),
                "#94e2d5".into(),
            ],
        }
    }

    pub fn catppuccin_latte() -> ThemeConfig {
        ThemeConfig {
            name: "Catppuccin Latte".into(),
            mode: ThemeMode::Light,
            bg_primary: "#eff1f5".into(),
            bg_secondary: "#e6e9ef".into(),
            bg_surface: "#ccd0da".into(),
            fg_primary: "#4c4f69".into(),
            fg_muted: "#9ca0b0".into(),
            border_color: "#dce0e8".into(),
            accent: "#1e66f5".into(),
            diff_add_bg: "#d5eecf".into(),
            diff_add_fg: "#40a02b".into(),
            diff_del_bg: "#fadad7".into(),
            diff_del_fg: "#d20f39".into(),
            diff_mod_bg: "#fcf2c5".into(),
            diff_mod_fg: "#df8e1d".into(),
            conflict_bg: "#fae5cf".into(),
            conflict_border: "#fe640b".into(),
            branch_lanes: vec![
                "#1e66f5".into(),
                "#8839ef".into(),
                "#d20f39".into(),
                "#fe640b".into(),
                "#40a02b".into(),
                "#179299".into(),
            ],
        }
    }

    pub fn one_dark_pro() -> ThemeConfig {
        ThemeConfig {
            name: "One Dark Pro".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#282c34".into(),
            bg_secondary: "#21252b".into(),
            bg_surface: "#2c313a".into(),
            fg_primary: "#abb2bf".into(),
            fg_muted: "#5c6370".into(),
            border_color: "#1e1e24".into(),
            accent: "#61afef".into(),
            diff_add_bg: "#233827".into(),
            diff_add_fg: "#98c379".into(),
            diff_del_bg: "#3c2025".into(),
            diff_del_fg: "#e06c75".into(),
            diff_mod_bg: "#353222".into(),
            diff_mod_fg: "#e5c07b".into(),
            conflict_bg: "#3e301f".into(),
            conflict_border: "#d19a66".into(),
            branch_lanes: vec![
                "#61afef".into(),
                "#c678dd".into(),
                "#98c379".into(),
                "#e5c07b".into(),
                "#e06c75".into(),
                "#56b6c2".into(),
            ],
        }
    }

    pub fn rose_pine() -> ThemeConfig {
        ThemeConfig {
            name: "Rose Pine".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#191724".into(),
            bg_secondary: "#1f1d2e".into(),
            bg_surface: "#26233a".into(),
            fg_primary: "#e0def4".into(),
            fg_muted: "#6e6a86".into(),
            border_color: "#14121d".into(),
            accent: "#ebbcba".into(),
            diff_add_bg: "#203332".into(),
            diff_add_fg: "#9ccfd8".into(),
            diff_del_bg: "#3d212c".into(),
            diff_del_fg: "#eb6f92".into(),
            diff_mod_bg: "#383526".into(),
            diff_mod_fg: "#f6c177".into(),
            conflict_bg: "#3d2c2b".into(),
            conflict_border: "#ea9a97".into(),
            branch_lanes: vec![
                "#ebbcba".into(),
                "#31748f".into(),
                "#9ccfd8".into(),
                "#c4a7e7".into(),
                "#f6c177".into(),
                "#eb6f92".into(),
            ],
        }
    }

    pub fn solarized_dark() -> ThemeConfig {
        ThemeConfig {
            name: "Solarized Dark".into(),
            mode: ThemeMode::Dark,
            bg_primary: "#002b36".into(),
            bg_secondary: "#073642".into(),
            bg_surface: "#0a4352".into(),
            fg_primary: "#839496".into(),
            fg_muted: "#586e75".into(),
            border_color: "#073642".into(),
            accent: "#268bd2".into(),
            diff_add_bg: "#0d3a2b".into(),
            diff_add_fg: "#859900".into(),
            diff_del_bg: "#3d1a24".into(),
            diff_del_fg: "#dc322f".into(),
            diff_mod_bg: "#373318".into(),
            diff_mod_fg: "#b58900".into(),
            conflict_bg: "#3a2818".into(),
            conflict_border: "#cb4b16".into(),
            branch_lanes: vec![
                "#268bd2".into(),
                "#2aa198".into(),
                "#859900".into(),
                "#b58900".into(),
                "#cb4b16".into(),
                "#d33682".into(),
            ],
        }
    }

    pub fn solarized_light() -> ThemeConfig {
        ThemeConfig {
            name: "Solarized Light".into(),
            mode: ThemeMode::Light,
            bg_primary: "#fdf6e3".into(),
            bg_secondary: "#eee8d5".into(),
            bg_surface: "#e4ddc8".into(),
            fg_primary: "#657b83".into(),
            fg_muted: "#93a1a1".into(),
            border_color: "#d5c4a1".into(),
            accent: "#268bd2".into(),
            diff_add_bg: "#e7f2cf".into(),
            diff_add_fg: "#859900".into(),
            diff_del_bg: "#fbe0dc".into(),
            diff_del_fg: "#dc322f".into(),
            diff_mod_bg: "#f7f0c5".into(),
            diff_mod_fg: "#b58900".into(),
            conflict_bg: "#f9e7d9".into(),
            conflict_border: "#cb4b16".into(),
            branch_lanes: vec![
                "#268bd2".into(),
                "#2aa198".into(),
                "#859900".into(),
                "#b58900".into(),
                "#cb4b16".into(),
                "#d33682".into(),
            ],
        }
    }
}

impl Default for EmbeddedThemeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ThemeStoragePort for EmbeddedThemeAdapter {
    async fn list_available_themes(&self) -> Result<Vec<String>, DomainError> {
        let mut list: Vec<String> = self.themes.keys().cloned().collect();
        list.sort();
        Ok(list)
    }

    async fn get_theme(&self, name: &str) -> Result<ThemeConfig, DomainError> {
        self.themes
            .get(name)
            .cloned()
            .ok_or_else(|| DomainError::ConfigurationError(format!("Tema '{name}' no encontrado")))
    }
}
