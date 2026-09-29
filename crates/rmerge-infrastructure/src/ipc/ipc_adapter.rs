use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use rmerge_domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    OpenRepository { path: String },
    OpenMergeTool {
        base: String,
        local: String,
        remote: String,
        output: String,
    },
    Blame { file: String, line: Option<u32> },
    Search { query: String },
}

pub struct IpcChannel;

impl IpcChannel {
    pub fn socket_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            PathBuf::from(r"\\.\pipe\rmerge-ipc")
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
                PathBuf::from(runtime_dir).join("rmerge.sock")
            } else {
                let uid = unsafe { libc::getuid() };
                PathBuf::from(format!("/tmp/rmerge-{uid}.sock"))
            }
        }
    }

    /// Intenta enviar un mensaje a una instancia existente de rmerge en segundo plano
    pub async fn send_to_active_instance(message: &IpcMessage) -> Result<(), DomainError> {
        #[cfg(not(target_os = "windows"))]
        {
            let path = Self::socket_path();
            if !path.exists() {
                return Err(DomainError::Internal("No hay instancia activa de rmerge".into()));
            }

            let mut stream = match tokio::net::UnixStream::connect(&path).await {
                Ok(s) => s,
                Err(_) => {
                    // Socket obsoleto/muerto sin proceso escuchando: limpiarlo
                    let _ = std::fs::remove_file(&path);
                    return Err(DomainError::Internal("Instancia previa no respondía; socket limpiado".into()));
                }
            };

            let payload = serde_json::to_vec(message)
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            stream.write_all(&payload).await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            Ok(())
        }
        #[cfg(target_os = "windows")]
        {
            use tokio::net::windows::named_pipe::ClientOptions;
            let path = Self::socket_path();
            let mut client = ClientOptions::new()
                .open(&path)
                .map_err(|e| DomainError::Internal(format!("No se pudo conectar al named pipe: {e}")))?;

            let payload = serde_json::to_vec(message)
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            client.write_all(&payload).await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            Ok(())
        }
    }
}
