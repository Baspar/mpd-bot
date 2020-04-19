use crate::utils::BoxError;
use std::sync::Arc;
use std::sync::Mutex;
use rusqlite::{params, Connection, Result};
use std::path::Path;

fn migrate(conn: &Connection) -> Result<(), BoxError> {
    conn.execute("CREATE TABLE IF NOT EXISTS chat_authorization (
        chat_id TEXT PRIMARY KEY,
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

pub async fn is_chat_authorized(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<bool, BoxError> {
    tokio::task::spawn_blocking(move || -> Result<bool, BoxError> {
        let conn = conn.lock().unwrap();
        let mut query = conn.prepare("SELECT * from chat_authorization where chat_id = ?1;")?;
        let row: Result<bool, _> = query.query_row(params![chat_id], |row| row.get(3));
        match row {
            Ok(is_authorized) => return Ok(is_authorized),
            _ => {
                println!("New chat: {}", chat_id);
                conn.execute("INSERT INTO chat_authorization (chat_id, authorized) VALUES (?, 0)", params![chat_id])?;
            }
        }
        Ok(false)
    }).await?
}
