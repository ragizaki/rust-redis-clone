mod parser;

use anyhow::Result;
use parser::{Parser, Payload};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            match handle_connection(stream).await {
                Ok(()) => (),
                Err(msg) => eprintln!("Error handling connection: {}", msg),
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = [0; 1024];
    loop {
        let num_bytes = stream.read(&mut buf).await?;

        if num_bytes == 0 {
            return Ok(());
        }

        let request = std::str::from_utf8(&buf[..num_bytes]).expect("Invalid ASCII");
        let parser = Parser::new();
        let body = parser.parse(request)?;
        let payload = Payload::from_array(body)?;

        stream.write_all(payload.serialize().as_bytes()).await?;
    }
}
