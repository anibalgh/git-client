use rmerge_domain::entities::GitCredential;
use rmerge_domain::errors::DomainError;

#[async_trait::async_trait]
pub trait CredentialStoragePort: Send + Sync {
    /// Obtiene las credenciales guardadas para un host dado (ej. "github.com")
    async fn get_credential(&self, host: &str) -> Result<Option<GitCredential>, DomainError>;

    /// Guarda o actualiza las credenciales para un host de forma segura y encriptada
    async fn save_credential(&self, credential: &GitCredential) -> Result<(), DomainError>;

    /// Elimina las credenciales asociadas a un host
    async fn delete_credential(&self, host: &str) -> Result<(), DomainError>;

    /// Lista todos los hosts que tienen credenciales configuradas
    async fn list_hosts(&self) -> Result<Vec<String>, DomainError>;
}
