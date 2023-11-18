use anyhow::Error;
use std::{io::Write, net::TcpStream};

use crate::parser::{Value, ARRAY, BULK, ERROR, INTEGER, STRING};

pub struct RespWriter {
    stream: TcpStream,
}

impl RespWriter {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    fn marshal_value(value: Value) -> Vec<u8> {
        match value {
            Value::Str(s) => format!("{}{}\r\n", STRING, s).into_bytes(),
            Value::Num(n) => format!("{}{}\r\n", INTEGER, n).into_bytes(),
            Value::Bulk(b) => format!("{}{}\r\n{}\r\n", BULK, b.len(), b).into_bytes(),
            Value::Array(a) => {
                let mut result = format!("{}{}\r\n", ARRAY, a.len()).into_bytes();
                for v in a {
                    result.extend(Self::marshal_value(v).iter());
                }
                result
            }
            Value::Error(e) => format!("{}{}\r\n", ERROR, e).into_bytes(),
            Value::Null => "$-1\r\n".as_bytes().to_vec(),
        }
    }

    pub fn write(&mut self, value: Value) -> Result<(), Error> {
        let bytes = Self::marshal_value(value);
        self.stream.write_all(&bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marshal_bulk_value() {
        assert_eq!(
            RespWriter::marshal_value(Value::Bulk("hello".to_string())),
            b"$5\r\nhello\r\n"
        );
    }

    #[test]
    fn marshal_array_value() {
        assert_eq!(
            RespWriter::marshal_value(Value::Array(vec![
                Value::Bulk("hello".to_string()),
                Value::Bulk("world".to_string()),
            ])),
            b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }
}
