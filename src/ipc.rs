use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    ToggleStartMenu,
    ToggleControlCenter,
    ReloadConfig,
    ShowOsd(String),
    SnapActive(String),
    Quit,
}

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("kepler-shell.sock")
    } else {
        std::env::temp_dir().join(format!("kepler-shell-{}.sock", users_id()))
    }
}

fn users_id() -> u32 {
    unsafe { libc::getuid() }
}

pub struct IpcServer;

impl IpcServer {
    pub fn start(sender: async_channel::Sender<IpcCommand>) {
        let sock_path = get_socket_path();
        let _ = std::fs::remove_file(&sock_path);

        tokio::spawn(async move {
            let listener = match UnixListener::bind(&sock_path) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[Kepler IPC] Failed to bind socket {:?}: {}", sock_path, e);
                    return;
                }
            };

            loop {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut buf = vec![0u8; 1024];
                    if let Ok(n) = stream.read(&mut buf).await {
                        if n > 0 {
                            if let Ok(cmd) = serde_json::from_slice::<IpcCommand>(&buf[..n]) {
                                let _ = sender.send(cmd).await;
                                let _ = stream.write_all(b"OK\n").await;
                            }
                        }
                    }
                }
            }
        });
    }
}

pub struct IpcClient;

impl IpcClient {
    pub async fn send_command(cmd: IpcCommand) -> Result<(), String> {
        let sock_path = get_socket_path();
        let mut stream = UnixStream::connect(&sock_path)
            .await
            .map_err(|e| format!("Could not connect to Kepler daemon: {}", e))?;

        let data = serde_json::to_vec(&cmd).map_err(|e| e.to_string())?;
        stream.write_all(&data).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
