use crate::{parser::Value, Db};

fn ping(x: Vec<Value>, _: Db) -> Value {
    if x.is_empty() {
        Value::Str("PONG".to_string())
    } else {
        Value::Str(x[0].unwrap_str())
    }
}

fn unknown(x: Vec<Value>, _: Db) -> Value {
    Value::Error(format!("Unknown command: {}", x[0].unwrap_str()))
}

fn set(x: Vec<Value>, db: Db) -> Value {
    if x.len() == 2 {
        let mut db = db.lock().unwrap();
        db.insert(x[0].unwrap_str(), x[1].unwrap_str());
        Value::Str("OK".to_string())
    } else {
        Value::Error("Wrong number of arguments".to_string())
    }
}

fn get(x: Vec<Value>, db: Db) -> Value {
    if x.len() == 1 {
        let db = db.lock().unwrap();
        match db.get(&x[0].unwrap_str()) {
            Some(value) => Value::Bulk(value.to_string()),
            None => Value::Null,
        }
    } else {
        Value::Error("Wrong number of arguments".to_string())
    }
}

pub fn handler(value: &Value) -> impl Fn(Vec<Value>, Db) -> Value {
    // println!("{:#?}", value);
    if let Value::Bulk(command) = value {
        match command.to_lowercase().as_str() {
            "ping" => ping,
            "set" => set,
            "get" => get,
            _ => unknown,
        }
    } else {
        unknown
    }
}
