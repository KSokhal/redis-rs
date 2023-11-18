use anyhow::Result;
// use parser::Value;
use std::io::{BufReader, Write};
use std::net::TcpListener;

use crate::parser::RespReader;

mod parser;

// fn marshal_value(value: Value) -> [u8] {
//     match value.content {
//         Value::Str(s) => format!("+{}\r\n", s).into_bytes(),
//         Value::Num(n) => format!(":{}\r\n", n).into_bytes(),
//         Value::Bulk(b) => format!("${}\r\n{}\r\n", b.len(), b).into_bytes(),
//         Value::Array(a) => {
//             let mut result = format!("*{}\r\n", a.len()).into_bytes();
//             for v in a {
//                 result.extend(marshal_value(v).iter());
//             }
//             result
//         }
//     }
// }

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
