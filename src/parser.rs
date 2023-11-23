use anyhow::Error;
use std::{io::Read, net::TcpStream};

pub(crate) const STRING: char = '+';
pub(crate) const ERROR: char = '-';
pub(crate) const INTEGER: char = ':';
pub(crate) const BULK: char = '$';
pub(crate) const ARRAY: char = '*';

#[allow(dead_code)]
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

    #[allow(dead_code)]
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

fn read_char(stream: &mut TcpStream) -> Result<char, Error> {
    let mut char: [u8; 1] = [0; 1];
    stream.read_exact(&mut char)?;
    Ok(char[0] as char)
}

fn read_int(stream: &mut TcpStream) -> Result<usize, Error> {
    let mut num: [u8; 1] = [0; 1];
    stream.read_exact(&mut num)?;
    let n = (num[0] as char)
        .to_digit(10)
        .expect("Failed to read int from buffer");
    Ok(n as usize)
}

fn read_str(stream: &mut TcpStream, size: usize) -> Result<String, Error> {
    let mut buf = vec![0; size];
    stream.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn consume_newline(stream: &mut TcpStream) -> Result<(), Error> {
    stream.read_exact(&mut [0; 2])?;
    Ok(())
}

pub fn parse(stream: &mut TcpStream) -> Result<Value, Error> {
    let _type = read_char(stream)?;

    match _type {
        ARRAY => {
            let arr_len = read_int(stream)?;

            let mut value_arr: Vec<Value> = Vec::new();

            for _ in 0..arr_len {
                consume_newline(stream)?;
                let v = parse(stream)?;
                value_arr.push(v);
            }

            Ok(Value::Array(value_arr))
        }
        BULK => {
            let str_len = read_int(stream)?;
            consume_newline(stream)?;

            let string = read_str(stream, str_len)?;

            Ok(Value::Bulk(string))
        }
        _ => panic!("unknown type {}", _type),
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, net::TcpListener, thread};

    use super::*;

    #[test]
    fn parse_resp() {
        let input = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";

        // Start a mock server in a new thread
        let handle = thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.write_all(input).unwrap();
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        });

        // Give the server a moment to start up
        thread::sleep(std::time::Duration::from_secs(1));

        let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

        let expected_value = Value::Array(vec![
            Value::Bulk("hello".to_string()),
            Value::Bulk("world".to_string()),
        ]);

        assert_eq!(parse(&mut stream).unwrap(), expected_value);

        // Clean up the server thread
        handle.join().unwrap();
    }
}
