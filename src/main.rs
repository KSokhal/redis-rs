use anyhow::Result;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use crate::handler::handler;
use crate::parser::RespReader;
use crate::writer::RespWriter;

mod handler;
mod parser;
mod writer;

type Db = Arc<Mutex<HashMap<String, String>>>;

fn main() -> Result<()> {
    // HashMap that be shared between threads
    // Mutex is used to ensure that only one thread can access the HashMap at a time, by locking and unlocking the value
    // Arc is used to allow the HashMap is shared between threads
    // Atomic reference counting (Arc) counts the number of reference that point to the same memory heap
    // It is thread-safe since is uses atomic operations to increment and decrement the reference count, however they are more expensive
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());

                // let reader = BufReader::new(&mut stream);
                let mut parser = RespReader::new(&mut stream);

                let value = parser.parse()?;

                let values_arr = value.unwrap_arr();

                let response = handler(&values_arr[0])(values_arr[1..].to_vec(), db.clone());

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
