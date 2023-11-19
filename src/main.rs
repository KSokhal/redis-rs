use anyhow::Result;
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
                // let reader = BufReader::new(&mut stream);
                let mut parser = RespReader::new(&mut stream);

                let value = parser.parse()?;

                let values_arr = value.unwrap_arr();

                let response = handler(values_arr, &datastore);

                let mut writer = RespWriter::new(stream);
                writer.write(response)?;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}
