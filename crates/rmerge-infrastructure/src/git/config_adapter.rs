use std::path::{Path, PathBuf};
use git2::{Config, ConfigLevel, Repository};
use rmerge_application::ports::out::GitConfigPort;
use rmerge_domain::entities::{ConfigScope, GitAuthor};
use rmerge_domain::errors::DomainError;

pub struct GitConfigAdapter;

impl GitConfigAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitConfigAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitConfigPort for GitConfigAdapter {
    async fn get_effective_author(&self, repo_path: &Path) -> Result<Option<GitAuthor>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        
        let config = repo.config()
            .map_err(|e| DomainError::GitOperationFailed(format!("Error al leer configuración: {e}")))?;

        let name = config.get_string("user.name").ok();
        let email = config.get_string("user.email").ok();

        match (name, email) {
            (Some(n), Some(e)) if !n.trim().is_empty() && !e.trim().is_empty() => {
                Ok(Some(GitAuthor::new(n.trim(), e.trim())))
            }
            _ => Ok(None),
        }
    }

    async fn get_local_author(&self, repo_path: &Path) -> Result<Option<GitAuthor>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        
        let config = repo.config()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        // Abrir específicamente el nivel local
        if let Ok(local_cfg) = config.open_level(ConfigLevel::Local) {
            let name = local_cfg.get_string("user.name").ok();
            let email = local_cfg.get_string("user.email").ok();
            if let (Some(n), Some(e)) = (name, email) {
                if !n.trim().is_empty() && !e.trim().is_empty() {
                    return Ok(Some(GitAuthor::new(n.trim(), e.trim())));
                }
            }
        }

        Ok(None)
    }

    async fn get_global_author(&self) -> Result<Option<GitAuthor>, DomainError> {
        let config = Config::open_default()
            .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

        if let Ok(global_cfg) = config.open_level(ConfigLevel::Global) {
            let name = global_cfg.get_string("user.name").ok();
            let email = global_cfg.get_string("user.email").ok();
            if let (Some(n), Some(e)) = (name, email) {
                if !n.trim().is_empty() && !e.trim().is_empty() {
                    return Ok(Some(GitAuthor::new(n.trim(), e.trim())));
                }
            }
        }

        Ok(None)
    }

    async fn set_author(
        &self,
        repo_path: &Path,
        author: &GitAuthor,
        scope: ConfigScope,
    ) -> Result<(), DomainError> {
        match scope {
            ConfigScope::Local => {
                let repo = Repository::open(repo_path)
                    .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
                let mut config = repo.config()
                    .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
                
                config.set_str("user.name", &author.name)
                    .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
                config.set_str("user.email", &author.email)
                    .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            }
            ConfigScope::Global => {
                let global_config_path: PathBuf = dirs::home_dir()
                    .ok_or_else(|| DomainError::ConfigurationError("No se encontró el directorio de inicio (HOME)".into()))?
                    .join(".gitconfig");

                let mut config = Config::open(&global_config_path)
                    .or_else(|_| Config::new())
                    .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;

                config.set_str("user.name", &author.name)
                    .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;
                config.set_str("user.email", &author.email)
                    .map_err(|e| DomainError::ConfigurationError(e.to_string()))?;
            }
        }

        Ok(())
    }
}
