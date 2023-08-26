use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

pub struct BoltAgent {
    ct: CancellationToken,
    stream: TcpStream,
    state: AgentState,
}

struct AgentState {
    authenticated: bool,
}

impl BoltAgent {
    pub fn new(stream: TcpStream, ct: CancellationToken) -> BoltAgent {
        Self {
            stream,
            ct,
            state: AgentState {
                authenticated: false,
            },
        }
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        let token = self.ct.clone();
        let result = tokio::select! {
            v = self.handshake() => {
                v
            },
            _ = token.cancelled() => {
                Err(std::io::Error::from(ErrorKind::ConnectionAborted))
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                Err(std::io::Error::from(ErrorKind::InvalidInput))
            }
        };
        if self.ct.is_cancelled() {
            return Err(Error::from(ErrorKind::ConnectionAborted));
        }

        Ok(())
    }

    async fn handshake(&mut self) -> std::io::Result<(u8, u8)> {
        let mut buffer = [0u8; 20];
        self.stream.read_exact(&mut buffer).await?;
        if !buffer[..4].iter().eq(&[60u8, 60u8, 80u8, 17u8]) {
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        let version = buffer[4..]
            .chunks(4)
            .into_iter()
            .map(|v| {
                let major = v[3];
                let minor = v[2];
                let _ = v[1];
                return (major, minor);
            })
            .rev()
            .last()
            .unwrap();

        self.stream
            .write_all(&[0u8, 0u8, version.1, version.0])
            .await?;
        Ok(version)
    }
}
