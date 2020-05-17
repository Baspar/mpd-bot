use regex::Regex;
use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::db;
use crate::telegram;
use crate::utils::{BoxError,read_entity_from_text};
use crate::telegram::structs::MessageEntity;
use crate::action;

pub async fn authorize(conn: Arc<Mutex<Connection>>, chat_id: i64, text: String) -> Result<(), BoxError> {
    let re = Regex::new(r"\d*").unwrap();
    Ok(())
}

