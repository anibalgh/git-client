use rmerge_domain::errors::DomainError;
use rmerge_domain::value_objects::{CodeFontSetting, FontSetting, TypographyConfig};

#[derive(Debug, Clone)]
pub struct AvailableFonts {
    pub system_fonts: Vec<String>,
    pub monospace_fonts: Vec<String>,
}

#[async_trait::async_trait]
pub trait ManageTypographyUseCase: Send + Sync {
    async fn get_available_fonts(&self) -> Result<AvailableFonts, DomainError>;
    async fn get_current_typography(&self) -> Result<TypographyConfig, DomainError>;
    async fn update_ui_font(&self, font: FontSetting) -> Result<(), DomainError>;
    async fn update_code_font(&self, font: CodeFontSetting) -> Result<(), DomainError>;
    async fn load_font_bytes(&self, family_name: &str) -> Result<Option<Vec<u8>>, DomainError>;
}
