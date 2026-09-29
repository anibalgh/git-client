use rmerge_domain::entities::GitCredential;
use rmerge_domain::errors::DomainError;

#[async_trait::async_trait]
pub trait ManageCredentialsUseCase: Send + Sync {
    /// Obtiene las credenciales para un host o extrae el host a partir de una URL
    async fn find_credential(&self, host_or_url: &str) -> Result<Option<GitCredential>, DomainError>;

    /// Guarda credenciales asociadas a un host o extrae el host de una URL
    async fn store_credential(&self, host_or_url: &str, username: &str, secret: &str) -> Result<(), DomainError>;

    /// Elimina credenciales asociadas a un host
    async fn remove_credential(&self, host_or_url: &str) -> Result<(), DomainError>;

    /// Lista los hosts configurados
    async fn get_configured_hosts(&self) -> Result<Vec<String>, DomainError>;
}
