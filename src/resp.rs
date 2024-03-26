use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq)]
pub struct Array {
    pub contents: Vec<BulkString>,
}

pub struct SimpleString(String);

#[derive(Debug, PartialEq)]
pub struct BulkString(pub String);

pub enum Payload {
    Simple(SimpleString),
    Bulk(BulkString),
}

impl Payload {
    pub fn serialize(&self) -> String {
        match self {
            Self::Simple(SimpleString(s)) => format!("+{s}\r\n"),
            Self::Bulk(BulkString(s)) => format!("${}\r\n{s}\r\n", s.len()),
        }
    }
}

impl Payload {
    pub fn from_array(value: Array) -> Result<Payload> {
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
            other => Err(anyhow!("Command {other} is unimplemented")),
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
}
