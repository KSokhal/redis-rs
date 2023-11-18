use anyhow::Error;
use std::io::{BufReader, Read};

const STRING: char = '+';
const ERROR: char = '-';
const INTEGER: char = ':';
const BULK: char = '$';
const ARRAY: char = '*';

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Num(i32),
    Bulk(String),
    Array(Vec<Value>),
    Error(String),
    Null,
}

fn marshal_value(value: Value) -> Vec<u8> {
    match value {
        Value::Str(s) => format!("{}{}\r\n", STRING, s).into_bytes(),
        Value::Num(n) => format!("{}{}\r\n", INTEGER, n).into_bytes(),
        Value::Bulk(b) => format!("{}{}\r\n{}\r\n", BULK, b.len(), b).into_bytes(),
        Value::Array(a) => {
            let mut result = format!("{}{}\r\n", ARRAY, a.len()).into_bytes();
            for v in a {
                result.extend(marshal_value(v).iter());
            }
            result
        }
        Value::Error(e) => format!("{}{}\r\n", ERROR, e).into_bytes(),
        Value::Null => "$-1\r\n".as_bytes().to_vec(),
    }
}

pub struct RespReader<T: std::io::Read> {
    reader: BufReader<T>,
}

impl<T: std::io::Read> RespReader<T> {
    pub fn new(reader: BufReader<T>) -> Self {
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

    #[test]
    fn marshal_bulk_value() {
        assert_eq!(
            marshal_value(Value::Bulk("hello".to_string())),
            b"$5\r\nhello\r\n"
        );
    }

    #[test]
    fn marshal_array_value() {
        assert_eq!(
            marshal_value(Value::Array(vec![
                Value::Bulk("hello".to_string()),
                Value::Bulk("world".to_string()),
            ])),
            b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }
}
