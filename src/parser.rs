use crate::{
    resp::{Array, BulkString, Payload, SimpleString},
    server::Server,
};
use anyhow::{anyhow, Result};

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    pub fn from_array(value: Array, server: &mut Server) -> Result<Payload> {
        let mut iter = value.contents.iter();
        let command = iter.next().unwrap();

        match command.0.to_lowercase().as_str() {
            "ping" => Ok(Payload::Simple(SimpleString(String::from("PONG")))),
            "echo" => {
                let echoed = iter
                    .map(|BulkString(s)| s.clone())
                    .collect::<Vec<String>>()
                    .join(" ");
                Ok(Payload::Bulk(BulkString(echoed)))
            }
            "set" => {
                server.set(iter);

                Ok(Payload::Simple(SimpleString(String::from("OK"))))
            }
            "get" => Ok(server.get(iter)),
            "info" => Ok(Payload::Bulk(BulkString(server.info()))),
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

                let word_start = length + 3_usize;
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
