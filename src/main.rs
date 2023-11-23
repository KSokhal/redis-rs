use anyhow::Result;
use parser::{parse, Value};
use std::net::TcpListener;
use std::sync::Arc;

use crate::handler::{handler, Datastore};
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
                let value = parse(&mut stream)?;

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

// Could convert RespReader to plain function to avoid having to create a struct, since only used in    main
