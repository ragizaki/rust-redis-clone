//! RESP Types
//!
//! Provides an abstraction for RESP Types. These include:
//! Arrays, SimpleStrings, BulkStrings

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
    Simple(SimpleString),
    Bulk(BulkString),
    Null,
}

impl Payload {
    pub fn serialize(&self) -> String {
        match self {
            Self::Simple(SimpleString(s)) => format!("+{s}\r\n"),
            Self::Bulk(BulkString(s)) => format!("${}\r\n{s}\r\n", s.len()),
            Self::Null => String::from("$-1\r\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple() {
        let ss = Payload::Simple(SimpleString(String::from("ping")));
        assert_eq!(ss.serialize(), String::from("+ping\r\n"));
    }

    #[test]
    fn test_serialize_bulk() {
        let bs = Payload::Bulk(BulkString(String::from("ping")));
        assert_eq!(bs.serialize(), String::from("$4\r\nping\r\n"));
    }

    #[test]
    fn test_serialize_null() {
        assert_eq!(Payload::Null.serialize(), String::from("$-1\r\n"));
    }
}
