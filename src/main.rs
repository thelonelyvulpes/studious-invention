use bolt_agent::BoltAgent;
use chrono::Utc;
use log::{info, warn, LevelFilter};
use std::io::Write;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

mod bolt_agent;
mod script_parser;
mod stub_engine;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::new()
        .format(|buf, record| {
            buf.style().set_color(env_logger::fmt::Color::White);
            writeln!(
                buf,
                "{} [{}] - {}",
                Utc::now().format("%+"),
                record.level(),
                record.args(),
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let ct = CancellationToken::new();
    let mut sigterm = signal(SignalKind::terminate())?;
    let listen_address = "127.0.0.1:6379";
    let listener = TcpListener::bind(listen_address).await?;
    info!("Started listener on: {}.", listen_address);

    let mut set: JoinSet<()> = JoinSet::new();
    tokio::select! {
        _ = sigterm.recv() => {
            warn!("SIGTERM Received.");
            ct.cancel();
            tokio::time::sleep(Duration::from_secs(20)).await;
        },
        _ = run_client_test() => {

        }
        _ = run_server(ct.child_token(), listener, &mut set) => {
            // this block is never called as once the cancellation happens, this will exit.
        }
    }

    warn!("Graceful shutdown timed out.");
    Ok(())
}

async fn run_server(
    ct: CancellationToken,
    listener: TcpListener,
    handles: &mut JoinSet<()>,
) -> std::io::Result<()> {
    while !ct.is_cancelled() {
        match listener.accept().await {
            Ok((conn, addr)) => {
                let conn_token = ct.clone();
                conn.set_nodelay(true)?;
                info!("Connected to client: {}.", addr);
                handles.spawn(async move {
                    let agent = BoltAgent::new(conn, conn_token);
                    agent.run().await.unwrap();
                });
            }
            Err(_) => {
                panic!("we failed to start somehow?")
            }
        }
    }
    Ok(())
}

async fn run_client_test() -> std::io::Result<()> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut x = TcpStream::connect("127.0.0.1:6379").await?;
    x.write_all(&[
        60u8, 60u8, 80u8, 17u8, 0u8, 3u8, 3u8, 5u8, 0u8, 2u8, 4u8, 4u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8,
    ])
    .await?;
    let mut buf = [0u8; 4];
    x.read_exact(&mut buf).await?;

    info!("client - - {:?}", buf);
    tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;
    Ok(())
}
