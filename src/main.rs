use std::io::prelude::*;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => stream
                .write_all("+PONG\r\n".as_bytes())
                .expect("Failed to PONG"),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
