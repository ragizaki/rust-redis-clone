use crate::resp::{Array, BulkString, Payload, SimpleString};
use anyhow::{anyhow, Result};
use core::slice::Iter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

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

pub struct Parser {
    cache: Arc<Mutex<HashMap<String, Entry>>>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_array(&mut self, value: Array) -> Result<Payload> {
        let mut iter = value.contents.iter();
        let command = iter.next().unwrap();

        match command.0.to_lowercase().as_str() {
            "ping" => Ok(Payload::Simple(SimpleString(String::from("PONG")))),
            "echo" => {
                let echoed = iter
                    .map(|BulkString(s)| s.clone())
                    .collect::<Vec<String>>()
                    .join(" ");
                Ok(Payload::Bulk(BulkString(String::from(echoed))))
            }
            "set" => {
                self.set(iter);

                Ok(Payload::Simple(SimpleString(String::from("OK"))))
            }
            "get" => Ok(self.get(iter)),
            "info" => Ok(Payload::Bulk(BulkString(String::from("role:master")))),
            other => Err(anyhow!("Command {other} is unimplemented")),
        }
    }

    pub fn parse(&self, s: &str) -> Result<Array> {
        let mut contents: Vec<BulkString> = Vec::new();
        let (num_elements, size) = self.extract_until_clrf(&s[1..]);
        let num_elements: usize = num_elements.parse()?;

        // skip the *, \r, \n, and size of num elements
        let mut cursor = 3 + size;
        for _ in 0..num_elements {
            let (str, length) = self.parse_bulk_string(&s[cursor..])?;
            contents.push(str);
            cursor += length;
        }
        Ok(Array::new(contents))
    }

    fn parse_bulk_string(&self, s: &str) -> Result<(BulkString, usize)> {
        let mut chars = s.chars();
        match chars.next().unwrap() {
            '$' => {
                let (size, length) = self.extract_until_clrf(&s[1..]);
                let size: usize = size
                    .parse()
                    .expect(&format!("Could not parse {size} into a usize"));

                let word_start = length + 3 as usize;
                let str = String::from_utf8(s[word_start..word_start + size].as_bytes().to_vec())
                    .expect("Could not form string");

                // 5 common characters ($, \r, \n, \r, \n)
                Ok((BulkString(str), 5 + length + size))
            }
            other => Err(anyhow!("Bulk string must start with $, not {other}")),
        }
    }

    fn extract_until_clrf<'a>(&self, s: &'a str) -> (&'a str, usize) {
        if let Some(idx) = s.find("\r\n") {
            let extracted = &s[..idx];
            (extracted, extracted.chars().count())
        } else {
            (s, s.chars().count())
        }
    }

    fn set(&mut self, mut iter: Iter<'_, BulkString>) {
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

    fn get(&self, mut iter: Iter<'_, BulkString>) -> Payload {
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
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_extract_until_clrf() {
        let str = "192\r\n";
        assert_eq!(Parser::new().extract_until_clrf(str), ("192", 3));
    }

    #[test]
    fn test_parse_bulk_string() {
        let str = "$10\r\nheyheyhey1\r\n";
        let expected = (BulkString(String::from("heyheyhey1")), str.chars().count());
        let actual = Parser::new().parse_bulk_string(str);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_parse() {
        let str = "*2\r\n$4\r\necho\r\n$3\r\nhey\r\n";
        let expected = Array::new(vec![
            BulkString(String::from("echo")),
            BulkString(String::from("hey")),
        ]);

        let actual = Parser::new().parse(str);
        assert_eq!(actual.unwrap(), expected);
    }
}
