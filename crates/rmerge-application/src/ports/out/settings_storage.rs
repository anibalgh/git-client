use rmerge_domain::errors::DomainError;
use rmerge_domain::value_objects::TypographyConfig;
use serde::{Deserialize, Serialize};

fn default_language() -> String {
    "es".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme_name: String,
    pub typography: TypographyConfig,
    pub auto_refresh: bool,
    pub recent_repos: Vec<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_name: "Sublime Dark".into(),
            typography: TypographyConfig::default(),
            auto_refresh: true,
            recent_repos: Vec::new(),
            language: default_language(),
        }
    }
}

#[async_trait::async_trait]
pub trait SettingsStoragePort: Send + Sync {
    async fn load_settings(&self) -> Result<AppSettings, DomainError>;
    async fn save_settings(&self, settings: &AppSettings) -> Result<(), DomainError>;
}
