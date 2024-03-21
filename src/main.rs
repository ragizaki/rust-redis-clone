use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("Client disconnected!");
                return;
            }
            Ok(_) => {
                stream
                    .write_all("+PONG\r\n".as_bytes())
                    .expect("Failed to PONG");
            }
            Err(err) => {
                eprintln!("Error reading from stream: {}", err);
                return;
            }
        }
    }
}
