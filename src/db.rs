use crate::utils::BoxError;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{params, Connection, Result};
use std::path::Path;

fn migrate(conn: &Connection) -> Result<(), BoxError> {
    conn.execute("CREATE TABLE IF NOT EXISTS chat_authorization (
        chat_id TEXT PRIMARY KEY,
        first_name Text,
        last_name Text,
        authorized NUMERIC
    )", params![])?;
    Ok(())
}

pub fn init() -> Result<Arc<Mutex<Connection>>, BoxError> {
    println!("Connecting to DB");
    let path = Path::new("db.sqlite");
    let conn = Connection::open(path)?;
    println!("Connected");
    println!("Migrating DB");
    migrate(&conn)?;
    println!("Migrated DB");
    let conn = Arc::new(Mutex::new(conn));
    Ok(conn)
}
