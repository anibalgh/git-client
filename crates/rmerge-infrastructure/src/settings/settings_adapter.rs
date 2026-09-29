use std::path::PathBuf;
use rmerge_application::ports::out::{AppSettings, SettingsStoragePort};
use rmerge_domain::errors::DomainError;

pub struct JsonSettingsAdapter {
    custom_path: Option<PathBuf>,
}

impl JsonSettingsAdapter {
    pub fn new() -> Self {
        Self { custom_path: None }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { custom_path: Some(path) }
    }

    fn resolve_path(&self) -> Result<PathBuf, DomainError> {
        if let Some(ref path) = self.custom_path {
            return Ok(path.clone());
        }

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rmerge");
        
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

        Ok(config_dir.join("settings.json"))
    }
}

impl Default for JsonSettingsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SettingsStoragePort for JsonSettingsAdapter {
    async fn load_settings(&self) -> Result<AppSettings, DomainError> {
        let path = self.resolve_path()?;
        if !path.exists() {
            let defaults = AppSettings::default();
            self.save_settings(&defaults).await?;
            return Ok(defaults);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

        let settings = serde_json::from_str(&content)
            .unwrap_or_else(|_| AppSettings::default());

        Ok(settings)
    }

    async fn save_settings(&self, settings: &AppSettings) -> Result<(), DomainError> {
        let path = self.resolve_path()?;
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

        std::fs::write(&path, json)
            .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

        Ok(())
    }
}
