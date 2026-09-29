use rmerge_application::ports::out::NetworkPort;
use rmerge_domain::errors::DomainError;

pub struct TokioNetworkAdapter;

impl TokioNetworkAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TokioNetworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NetworkPort for TokioNetworkAdapter {
    async fn check_connection(&self, host: &str, port: u16) -> Result<(), DomainError> {
        use std::time::Duration;
        use tokio::net::TcpStream;

        let target = format!("{host}:{port}");
        match tokio::time::timeout(Duration::from_millis(2500), TcpStream::connect(&target)).await {
            Ok(Ok(_stream)) => Ok(()),
            Ok(Err(e)) => Err(DomainError::NetworkError(format!(
                "Conexión rechazada o error TCP hacia {target}: {e}"
            ))),
            Err(_) => Err(DomainError::NetworkError(format!(
                "Timeout al intentar conectar hacia {target}"
            ))),
        }
    }
}
