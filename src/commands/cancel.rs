use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::db;
use crate::telegram;
use crate::utils::BoxError;

pub async fn cancel(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<(), BoxError> {
    db::set_chat_status(conn, chat_id, format!("wait_for_command"), None).await?;
    telegram::send_message(chat_id, format!("Task cancelled")).await?;
    Ok(())
}
