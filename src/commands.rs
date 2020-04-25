use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::db;
use crate::telegram;
use crate::utils::BoxError;
use crate::telegram::structs::Message;

pub async fn cancel(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<(), BoxError> {
    db::reset_chat_status(conn, chat_id).await?;
    telegram::send_message(chat_id, format!("Task cancelled")).await?;
    Ok(())
}

pub async fn download(conn: Arc<Mutex<Connection>>, message: Message) -> Result<(), BoxError> {
    Ok(())
}
