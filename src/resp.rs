//! RESP Types
//!
//! Provides an abstraction for RESP Types. These include:
//! Arrays, SimpleStrings, BulkStrings

use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Array {
    pub contents: Vec<BulkString>,
}

impl Array {
    pub fn new(contents: Vec<BulkString>) -> Self {
        Self { contents }
    }
}

#[derive(Debug, PartialEq)]
pub struct SimpleString(pub String);

#[derive(Debug, PartialEq)]
pub struct BulkString(pub String);

#[derive(Debug, PartialEq)]
pub enum Payload {
    Array(Array),
    Simple(SimpleString),
    Bulk(BulkString),
    Null,
}

impl Payload {
    pub fn serialize(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl ToString for Payload {
    fn to_string(&self) -> String {
        match self {
            Self::Array(Array { contents }) => {
                let strings = contents
                    .iter()
                    .fold(String::new(), |mut acc, BulkString(s)| {
                        let _ = write!(acc, "${}\r\n{}\r\n", s.len(), s);
                        acc
                    });

                format!("*{}\r\n{}", contents.len(), strings)
            }
            Self::Simple(SimpleString(s)) => format!("+{s}\r\n"),
            Self::Bulk(BulkString(s)) => format!("${}\r\n{s}\r\n", s.len()),
            Self::Null => String::from("$-1\r\n"),
        }
    }
}

impl FromStr for Payload {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let contents = s
            .split_whitespace()
            .map(|s| BulkString(s.to_string()))
            .collect::<Vec<BulkString>>();

        Ok(Payload::Array(Array { contents }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple() {
        let ss = Payload::Simple(SimpleString(String::from("ping")));
        assert_eq!(ss.to_string(), String::from("+ping\r\n"));
    }

    #[test]
    fn test_serialize_bulk() {
        let bs = Payload::Bulk(BulkString(String::from("ping")));
        assert_eq!(bs.to_string(), String::from("$4\r\nping\r\n"));
    }

    #[test]
    fn test_serialize_null() {
        assert_eq!(Payload::Null.to_string(), String::from("$-1\r\n"));
    }
}
