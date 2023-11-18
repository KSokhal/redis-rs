use anyhow::Error;
use std::io::{BufReader, Read};

pub(crate) const STRING: char = '+';
pub(crate) const ERROR: char = '-';
pub(crate) const INTEGER: char = ':';
pub(crate) const BULK: char = '$';
pub(crate) const ARRAY: char = '*';

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Str(String),
    Num(i32),
    Bulk(String),
    Array(Vec<Value>),
    Error(String),
    Null,
}

impl Value {
    pub fn unwrap_str(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Bulk(b) => b.clone(),
            Value::Error(e) => e.clone(),
            _ => panic!("not a string"),
        }
    }

    pub fn unwrap_num(&self) -> i32 {
        match self {
            Value::Num(n) => *n,
            _ => panic!("not a number"),
        }
    }

    pub fn unwrap_arr(&self) -> Vec<Value> {
        match self {
            Value::Array(a) => a.clone(),
            _ => panic!("not an array"),
        }
    }
}

pub struct RespReader<T: std::io::Read> {
    reader: BufReader<T>,
}

impl<T: std::io::Read> RespReader<T> {
    pub fn new(stream: T) -> Self {
        let reader = BufReader::new(stream);
        Self { reader }
    }

    fn read_char(&mut self) -> Result<char, Error> {
        let mut char: [u8; 1] = [0; 1];
        self.reader.read_exact(&mut char)?;
        Ok(char[0] as char)
    }

    fn read_int(&mut self) -> Result<usize, Error> {
        let mut num: [u8; 1] = [0; 1];
        self.reader.read_exact(&mut num)?;
        let n = (num[0] as char)
            .to_digit(10)
            .expect("Failed to read int from buffer");
        Ok(n as usize)
    }

    fn read_str(&mut self, size: usize) -> Result<String, Error> {
        let mut buf = vec![0; size];
        self.reader.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn consume_newline(&mut self) -> Result<(), Error> {
        self.reader.read_exact(&mut [0; 2])?;
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Value, Error> {
        let _type = self.read_char()?;

        match _type {
            ARRAY => {
                let arr_len = self.read_int()?;

                let mut value_arr: Vec<Value> = Vec::new();

                for _ in 0..arr_len {
                    self.consume_newline()?;
                    let v = self.parse()?;
                    value_arr.push(v);
                }

                Ok(Value::Array(value_arr))
            }
            BULK => {
                let str_len = self.read_int()?;
                self.consume_newline()?;

                let string = self.read_str(str_len)?;

                Ok(Value::Bulk(string))
            }
            _ => panic!("unknown type {}", _type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resp() {
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";

        let mut parser = RespReader::new(BufReader::new(input.as_bytes()));

        let expected_value = Value::Array(vec![
            Value::Bulk("hello".to_string()),
            Value::Bulk("world".to_string()),
        ]);

        assert_eq!(parser.parse().unwrap(), expected_value);
    }
}
