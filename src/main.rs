mod parser;
mod resp;

use anyhow::Result;
use parser::Parser;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();

    // skip prog name
    args.next();

    let port = match args.next() {
        Some(arg) => arg,
        None => "6379".to_string(),
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let parser = Parser::new();
            match handle_connection(stream, parser).await {
                Ok(()) => (),
                Err(msg) => eprintln!("Error handling connection: {}", msg),
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream, mut parser: Parser) -> Result<()> {
    let mut buf = [0; 1024];
    loop {
        let num_bytes = stream.read(&mut buf).await?;

        if num_bytes == 0 {
            return Ok(());
        }

        let request = std::str::from_utf8(&buf[..num_bytes]).expect("Invalid ASCII");
        let body = parser.parse(request)?;
        let payload = parser.from_array(body)?;

        stream.write_all(payload.serialize().as_bytes()).await?;
    }
}
