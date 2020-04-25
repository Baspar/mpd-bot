use crate::utils::BoxError;
use std::sync::Arc;
use std::sync::Mutex;
use rusqlite::{params, Connection, Result};
use std::path::Path;

fn migrate(conn: &Connection) -> Result<(), BoxError> {
    conn.execute("CREATE TABLE IF NOT EXISTS chat_status (
        chat_id NUMERIC PRIMARY KEY,
        status STRING
    )", params![])?;
    conn.execute("CREATE TABLE IF NOT EXISTS chat_authorization (
        chat_id NUMERIC PRIMARY KEY,
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
        let row = conn
            .prepare("SELECT * from chat_authorization WHERE chat_id = ?1;")?
            .query_row(params![chat_id], |row| row.get(1));
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

pub async fn get_chat_status(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<String, BoxError> {
    tokio::task::spawn_blocking(move || -> Result<String, BoxError> {
        let conn = conn.lock().unwrap();
        let mut query = conn.prepare("SELECT * from chat_status WHERE chat_id = ?1;")?;
        let row: Result<String, _> = query.query_row(params![chat_id], |row| row.get(1));
        if let Ok(status) = row {
            return Ok(status)
        } else {
            conn.execute("INSERT INTO chat_status (chat_id, status) VALUES (?, ?)", params![chat_id, "wait_for_command"])?;
            return Ok(format!("wait_for_command"))
        }
    }).await?
}

pub async fn reset_chat_status(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<(), BoxError> {
    tokio::task::spawn_blocking(move || -> Result<(), BoxError> {
        let conn = conn.lock().unwrap();
        conn.execute("REPLACE INTO chat_status (chat_id, status) VALUES (?, ?)", params![chat_id, "wait_for_command"])?;
        Ok(())
    }).await?
}
