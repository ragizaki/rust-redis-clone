mod parser;
mod resp;
mod server;

use anyhow::Result;
use clap::Parser as ClapParser;
use parser::Parser;
use server::{Role, Server};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u64,

    #[arg(short, long, value_delimiter = ' ', num_args = 2)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let parser = Arc::new(Mutex::new(Parser::new()));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();

    let server = Server::new(match args.replicaof {
        Some(_) => Role::Slave,
        None => Role::Master,
    });

    if let Some(vec) = args.replicaof {
        let mut iter = vec.into_iter();
        let addr = iter.next().unwrap();
        let port = iter.next().unwrap();
        let stream = TcpStream::connect(format!("{addr}:{port}")).await?;
        send_handhshake(stream, &server, args.port).await?;
    }

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let cloned_parser = parser.clone();
        let server = server.clone();

        tokio::spawn(async move {
            match handle_connection(stream, cloned_parser, server).await {
                Ok(()) => (),
                Err(msg) => eprintln!("Error handling connection: {}", msg),
            }
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    parser: Arc<Mutex<Parser>>,
    mut server: Server,
) -> Result<()> {
    let mut buf = [0; 1024];
    loop {
        let num_bytes = stream.read(&mut buf).await?;

        if num_bytes == 0 {
            return Ok(());
        }

        let request = std::str::from_utf8(&buf[..num_bytes]).expect("Invalid ASCII");
        let parser = parser.lock().await;
        let body = parser.parse(request)?;
        let payload = Parser::from_array(body, &mut server)?;

        stream.write_all(&payload.serialize()).await?;
    }
}

async fn send_handhshake(mut stream: TcpStream, server: &Server, port: u64) -> Result<()> {
    let mut buf = [0; 1024];
    // PING Master
    let ping = server.payload("ping").unwrap();
    stream.write_all(&ping.serialize()).await?;
    stream.read(&mut buf).await?;

    // REPLCONF notifying master of listening port
    let msg = format!("REPLCONF listening-port {port}");
    let port_msg = server.payload(&msg).unwrap();
    stream.write_all(&port_msg.serialize()).await?;
    stream.read(&mut buf).await?;

    // REPLCONF notifying master of slave's capabilities
    let capa_msg = server.payload("REPLCONF capa psync2").unwrap();
    stream.write_all(&capa_msg.serialize()).await?;
    stream.read(&mut buf).await?;

    let psync_msg = server.payload("PSYNC ? -1").unwrap();
    stream.write_all(&psync_msg.serialize()).await?;
    stream.read(&mut buf).await?;

    Ok(())
}
