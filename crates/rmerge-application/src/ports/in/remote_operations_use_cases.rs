use rmerge_domain::errors::DomainError;
use std::path::Path;

#[async_trait::async_trait]
pub trait RemoteOperationsUseCase: Send + Sync {
    /// Parsea la URL remota buscando un host y puerto, verifica conectividad y ejecuta el comando git remoto.
    async fn execute_remote_command(
        &self,
        repo_path: &Path,
        remote_url: Option<String>,
        args: &[&str],
    ) -> Result<String, DomainError>;
}
