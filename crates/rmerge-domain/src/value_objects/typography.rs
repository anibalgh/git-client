use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontSetting {
    pub family: String,
    pub size_pt: f32,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeFontSetting {
    pub family: String,
    pub size_pt: f32,
    pub line_height: f32,
    pub enable_ligatures: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub ui_font: FontSetting,
    pub code_font: CodeFontSetting,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            ui_font: FontSetting {
                family: "Inter, Segoe UI, SF Pro Text, sans-serif".into(),
                size_pt: 12.0,
                weight: 400,
            },
            code_font: CodeFontSetting {
                family: "JetBrains Mono, Fira Code, Cascadia Code, monospace".into(),
                size_pt: 13.0,
                line_height: 1.45,
                enable_ligatures: true,
            },
        }
    }
}
