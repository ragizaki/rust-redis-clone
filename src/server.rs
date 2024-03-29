use crate::resp::{BulkString, Payload};
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use std::slice::Iter;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

pub struct Server {
    cache: Arc<Mutex<HashMap<String, Entry>>>,
    role: Role,
}

impl Server {
    pub fn new(role: Role) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
        }
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
}

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
