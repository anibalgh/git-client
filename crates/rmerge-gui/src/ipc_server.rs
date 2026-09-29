use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use rmerge_infrastructure::ipc::{IpcChannel, IpcMessage};

pub struct IpcServer;

impl IpcServer {
    pub fn start(event_sender: UnboundedSender<IpcMessage>) {
        tokio::spawn(async move {
            let socket_path = IpcChannel::socket_path();

            #[cfg(not(target_os = "windows"))]
            {
                // Limpiar socket anterior si existía
                if socket_path.exists() {
                    let _ = std::fs::remove_file(&socket_path);
                }

                if let Ok(listener) = tokio::net::UnixListener::bind(&socket_path) {
                    loop {
                        if let Ok((mut stream, _)) = listener.accept().await {
                            let mut buffer = Vec::new();
                            if stream.read_to_end(&mut buffer).await.is_ok() {
                                if let Ok(message) = serde_json::from_slice::<IpcMessage>(&buffer) {
                                    let _ = event_sender.send(message);
                                }
                            }
                        }
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                use tokio::net::windows::named_pipe::ServerOptions;
                loop {
                    if let Ok(mut server) = ServerOptions::new().first_pipe_instance(true).create(&socket_path) {
                        if server.connect().await.is_ok() {
                            let mut buffer = Vec::new();
                            if server.read_to_end(&mut buffer).await.is_ok() {
                                if let Ok(message) = serde_json::from_slice::<IpcMessage>(&buffer) {
                                    let _ = event_sender.send(message);
                                }
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        });
    }
}
