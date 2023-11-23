use anyhow::Result;
use parser::Value;
use std::net::TcpListener;
use std::sync::Arc;

use crate::handler::{handler, Datastore};
use crate::parser::RespReader;
use crate::writer::RespWriter;

mod handler;
mod parser;
mod writer;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    let datastore = Arc::new(Datastore::new());

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut parser = RespReader::new(&mut stream);

                let value = parser.parse()?;

                let mut writer = RespWriter::new(stream);

                if let Value::Array(values_arr) = value {
                    let response = handler(values_arr, &datastore);
                    writer.write(response)?;
                } else {
                    writer.write(Value::Error("Invalid command".to_string()))?;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}
