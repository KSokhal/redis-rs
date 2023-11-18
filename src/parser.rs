use anyhow::Error;
use std::io::{BufReader, Read};

const STRING: char = '+';
const ERROR: char = '-';
const INTEGER: char = ':';
const BULK: char = '$';
const ARRAY: char = '*';

#[derive(Debug, PartialEq, Eq)]
enum ValueContent {
    Str(String),
    Num(i32),
    Bulk(String),
    Array(Vec<Value>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value {
    _type: char,
    // str: Option<String>,
    // num: Option<i32>,
    // bulk: Option<String>,
    // array: Option<Vec<Value>>,
    content: ValueContent,
}

impl Value {
    fn new(_type: char, content: ValueContent) -> Self {
        Self {
            _type,
            // str: None,
            // num: None,
            // bulk: None,
            // array: None,
            content,
        }
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
                let mut value = Value::new(ARRAY, ValueContent::Array(value_arr));
                // value.array = Some(value_arr);
                Ok(value)
            }
            BULK => {
                let str_len = self.read_int()?;
                self.consume_newline()?;

                let string = self.read_str(str_len)?;

                let mut value = Value::new(BULK, ValueContent::Bulk(string));
                // value.bulk = Some(string);

                Ok(value)
            }
            _ => panic!("unknown type {}", _type),
        }
    }
}

#[test]
fn parse_resp() {
    let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";

    let mut parser = RespReader::new(BufReader::new(input.as_bytes()));

    let mut hello = Value::new(BULK, ValueContent::Bulk("hello".to_string()));
    // hello.bulk = Some("hello".to_string());

    let mut world = Value::new(BULK, ValueContent::Bulk("world".to_string()));
    // world.bulk = Some("world".to_string());

    let mut expected_value = Value::new(ARRAY, ValueContent::Array(vec![hello, world]));
    // expected_value.array = Some(vec![hello, world]);

    assert!(parser.parse().unwrap() == expected_value);
}
