mod parser;

use anyhow::{anyhow, Result};
use parser::RedisCommand;
use std::str::FromStr;
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

        let command = String::from_utf8(buf[..num_bytes].to_vec()).unwrap();
        let command = RedisCommand::from_str(&command);

        let response = match command {
            Ok(c) => match c {
                RedisCommand::Ping => String::from("+PONG\r\n"),
                RedisCommand::Echo(str) => format!("${}\r\n{}\r\n", str.len(), str),
            },
            Err(msg) => return Err(anyhow!(msg)),
        };

        stream.write_all(response.as_bytes()).await?;
    }
}
