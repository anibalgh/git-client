use std::sync::Arc;
use rmerge_domain::errors::DomainError;
use rmerge_domain::value_objects::{CodeFontSetting, FontSetting, TypographyConfig};
use crate::ports::r#in::{AvailableFonts, ManageTypographyUseCase};
use crate::ports::out::{FontDiscoveryPort, SettingsStoragePort};

pub struct TypographyService {
    font_discovery: Arc<dyn FontDiscoveryPort>,
    settings_storage: Arc<dyn SettingsStoragePort>,
}

impl TypographyService {
    pub fn new(
        font_discovery: Arc<dyn FontDiscoveryPort>,
        settings_storage: Arc<dyn SettingsStoragePort>,
    ) -> Self {
        Self {
            font_discovery,
            settings_storage,
        }
    }
}

#[async_trait::async_trait]
impl ManageTypographyUseCase for TypographyService {
    async fn get_available_fonts(&self) -> Result<AvailableFonts, DomainError> {
        let system_fonts = self.font_discovery.list_system_fonts().await?;
        let monospace_fonts = self.font_discovery.list_monospace_fonts().await?;

        Ok(AvailableFonts {
            system_fonts,
            monospace_fonts,
        })
    }

    async fn get_current_typography(&self) -> Result<TypographyConfig, DomainError> {
        let settings = self.settings_storage.load_settings().await?;
        Ok(settings.typography)
    }

    async fn update_ui_font(&self, font: FontSetting) -> Result<(), DomainError> {
        let mut settings = self.settings_storage.load_settings().await?;
        settings.typography.ui_font = font;
        self.settings_storage.save_settings(&settings).await
    }

    async fn update_code_font(&self, font: CodeFontSetting) -> Result<(), DomainError> {
        let mut settings = self.settings_storage.load_settings().await?;
        settings.typography.code_font = font;
        self.settings_storage.save_settings(&settings).await
    }

    async fn load_font_bytes(&self, family_name: &str) -> Result<Option<Vec<u8>>, DomainError> {
        self.font_discovery.load_font_bytes(family_name).await
    }
}
