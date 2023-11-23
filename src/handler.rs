use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::parser::Value;

// HashMap that be shared between threads
// Mutex is used to ensure that only one thread can access the HashMap at a time, by locking and unlocking the value
// Arc is used to allow the HashMap is shared between threads
// Atomic reference counting (Arc) counts the number of reference that point to the same memory heap
// It is thread-safe since is uses atomic operations to increment and decrement the reference count, however they are more expensive
// Not currently needed, since only one thread, but will be useful when we add concurrency

pub struct Datastore {
    db: Mutex<HashMap<String, String>>,
    hash_db: Mutex<HashMap<String, HashMap<String, String>>>,
}

impl Datastore {
    pub fn new() -> Self {
        Self {
            db: Mutex::new(HashMap::new()),
            hash_db: Mutex::new(HashMap::new()),
        }
    }

    pub fn set(&self, values: Vec<Value>) -> Value {
        if values.len() != 2 {
            return Value::Error("Wrong number of arguments".to_string());
        }
        let mut db = self.db.lock().unwrap();
        db.insert(values[0].unwrap_str(), values[1].unwrap_str());

        Value::Str("OK".to_string())
    }

    pub fn get(&self, values: Vec<Value>) -> Value {
        if values.len() != 1 {
            return Value::Error("Wrong number of arguments".to_string());
        }

        let db = self.db.lock().unwrap();

        match db.get(&values[0].unwrap_str()) {
            Some(value) => Value::Bulk(value.to_string()),
            None => Value::Null,
        }
    }

    pub fn hset(&self, values: Vec<Value>) -> Value {
        if values.len() != 3 {
            return Value::Error("Wrong number of arguments".to_string());
        }
        let mut hash_db = self.hash_db.lock().unwrap();
        let key = values[0].unwrap_str();
        let field = values[1].unwrap_str();
        let value = values[2].unwrap_str();

        match hash_db.get_mut(&key) {
            Some(map) => {
                map.insert(field.to_string(), value.to_string());
            }
            None => {
                let mut map = HashMap::new();
                map.insert(field.to_string(), value.to_string());
                hash_db.insert(key.to_string(), map);
            }
        }
        Value::Str("OK".to_string())
    }

    pub fn hget(&self, values: Vec<Value>) -> Value {
        if values.len() != 2 {
            return Value::Error("Wrong number of arguments".to_string());
        }
        let hash_db = self.hash_db.lock().unwrap();
        let key = values[0].unwrap_str();
        let field = values[1].unwrap_str();

        match hash_db.get(&key) {
            Some(map) => match map.get(&field) {
                Some(value) => Value::Bulk(value.to_string()),
                None => Value::Null,
            },
            None => Value::Null,
        }
    }

    pub fn hgetall(&self, values: Vec<Value>) -> Value {
        if values.len() != 1 {
            return Value::Error("Wrong number of arguments".to_string());
        }
        let hash_db = self.hash_db.lock().unwrap();
        let key = values[0].unwrap_str();

        match hash_db.get(&key) {
            Some(map) => {
                let mut resp = vec![];
                for (key, value) in map {
                    resp.push(Value::Bulk(key.to_string()));
                    resp.push(Value::Bulk(value.to_string()));
                }
                Value::Array(resp)
            }
            None => Value::Null,
        }
    }
}

fn ping(x: Vec<Value>) -> Value {
    if x.is_empty() {
        Value::Str("PONG".to_string())
    } else {
        Value::Str(x[0].unwrap_str())
    }
}

pub fn handler(values_arr: Vec<Value>, datastore: &Arc<Datastore>) -> Value {
    let (first, values) = values_arr.split_first().unwrap();

    if let Value::Bulk(command) = first {
        match command.to_lowercase().as_str() {
            "ping" => ping(values.to_vec()),
            "set" => datastore.set(values.to_vec()),
            "get" => datastore.get(values.to_vec()),
            "hset" => datastore.hset(values.to_vec()),
            "hget" => datastore.hget(values.to_vec()),
            "hgetall" => datastore.hgetall(values.to_vec()),
            _ => Value::Error(format!("Unknown command: {}", command)),
        }
    } else {
        Value::Error("Unknown value type".to_string())
    }
}
