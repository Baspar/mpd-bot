use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub fn init() {
    println!("Connecting to DB");
    let path = Path::new("db.sqlite");
    let conn = Connection::open(path);
    let _conn = Arc::new(Mutex::new(conn));
    println!("Connected");
}
