use anyhow::Result;
use std::io::{BufReader, Write};
use std::net::TcpListener;

use crate::parser::RespReader;

mod parser;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());

                let reader = BufReader::new(&mut stream);
                let mut parser = RespReader::new(reader);

                let value = parser.parse()?;

                println!("{:#?}", value);

                stream.write_all(b"+PONG\r\n")?;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}
