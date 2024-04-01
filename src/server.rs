use crate::resp::{BulkString, Payload};
use anyhow::{anyhow, Result};
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use std::slice::Iter;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, PartialEq)]
struct Entry {
    value: String,
    expiry: Option<Instant>,
}

impl Entry {
    fn new(value: String, expiry: Option<Instant>) -> Self {
        Self { value, expiry }
    }
}

#[derive(Clone)]
pub struct Server {
    cache: Arc<Mutex<HashMap<String, Entry>>>,
    pub role: Role,
}

impl Server {
    pub fn new(role: Role) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
        }
    }

    pub async fn receive_handshake(&self, stream: &mut TcpStream) -> Result<()> {
        match self.role {
            Role::Slave => Err(anyhow!("Replica cannot receive handshake")),
            Role::Master => {
                let mut buf = [0; 1024];
                let ok = self.payload("OK").unwrap();

                stream.read(&mut buf).await?;
                eprintln!("Received {:?} from the client", buf);
                stream.write_all(&ok.serialize()).await?;

                stream.read(&mut buf).await?;
                stream.write_all(&ok.serialize()).await?;

                Ok(())
            }
        }
    }

    pub async fn send_handshake(&self, mut stream: TcpStream, port: u64) -> Result<()> {
        let mut buf = [0; 1024];
        // PING Master
        let ping = self.payload("ping").unwrap();
        stream.write_all(&ping.serialize()).await?;
        stream.read(&mut buf).await?;

        // REPLCONF notifying master of listening port
        let msg = format!("REPLCONF listening-port {port}");
        let port_msg = self.payload(&msg).unwrap();
        stream.write_all(&port_msg.serialize()).await?;
        stream.read(&mut buf).await?;

        // REPLCONF notifying master of slave's capabilities
        let capa_msg = self.payload("REPLCONF capa psync2").unwrap();
        stream.write_all(&capa_msg.serialize()).await?;
        stream.read(&mut buf).await?;

        let psync_msg = self.payload("PSYNC ? -1").unwrap();
        stream.write_all(&psync_msg.serialize()).await?;
        stream.read(&mut buf).await?;

        Ok(())
    }

    pub fn set(&mut self, mut iter: Iter<'_, BulkString>) {
        let BulkString(key) = iter.next().unwrap();
        let BulkString(val) = iter.next().unwrap();
        let mut cache = self.cache.lock().unwrap();

        // if there is a next value, it is an expiry
        if let Some(BulkString(px)) = iter.next() {
            assert!(px == "px");
            let BulkString(expiry_str) = iter.next().unwrap();
            let expiry_ms: u64 = expiry_str.parse().expect("Could not parse expiry");
            let expiry = Instant::now() + Duration::from_millis(expiry_ms);
            let entry = Entry::new(val.to_string(), Some(expiry));
            cache.insert(key.to_string(), entry);
        } else {
            let entry = Entry::new(val.to_string(), None);
            cache.insert(key.to_string(), entry);
        }
    }

    pub fn get(&self, mut iter: Iter<'_, BulkString>) -> Payload {
        let BulkString(key) = iter.next().unwrap();
        let cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(key) {
            if let Some(expiry) = entry.expiry {
                if Instant::now() > expiry {
                    return Payload::Null;
                }
            }
            return Payload::Bulk(BulkString(entry.value.clone()));
        }

        Payload::Null
    }

    pub fn info(&self) -> String {
        let mut info = format!("role:{}", self.role.to_string());

        match self.role {
            Role::Master => {
                let replid = Alphanumeric.sample_string(&mut rand::thread_rng(), 40);
                info.push_str(&format!("master_replid:{replid}"));
                info.push_str("master_repl_offset:0");
            }
            Role::Slave => {}
        };

        info
    }

    pub fn payload(&self, s: &str) -> Option<Payload> {
        let msg = Payload::from_str(s).unwrap();
        Some(msg)
    }
}

#[derive(Clone, PartialEq)]
pub enum Role {
    Master,
    Slave,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Self::Master => String::from("master"),
            Self::Slave => String::from("slave"),
        }
    }
}
