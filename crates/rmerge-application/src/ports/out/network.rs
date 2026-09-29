use rmerge_domain::errors::DomainError;

#[async_trait::async_trait]
pub trait NetworkPort: Send + Sync {
    /// Verifica conectividad no bloqueante. Lanza DomainError::NetworkError en timeout/falla.
    async fn check_connection(&self, host: &str, port: u16) -> Result<(), DomainError>;
}
