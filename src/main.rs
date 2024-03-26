mod parser;
mod resp;

use anyhow::Result;
use clap::Parser as ClapParser;
use parser::Parser;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))
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
