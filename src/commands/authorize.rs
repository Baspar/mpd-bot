use regex::Regex;
use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::db;
use crate::telegram;
use crate::utils::BoxError;

pub async fn authorize(conn: Arc<Mutex<Connection>>, chat_id: i64, text: String) -> Result<(), BoxError> {
    let re = Regex::new(r"(\d+)").unwrap();
    if db::is_chat_admin(conn.clone(), chat_id).await? {
        if let Some(caps) = re.captures(&text) {
            if let Some(id) = caps.get(0) {
                db::set_chat_authorized(conn.clone(), id.as_str().to_string()).await?;
                telegram::send_message(chat_id, format!("{} is now authorized", id.as_str())).await?;
                return Ok(())
            }
        }
        db::set_chat_status(conn.clone(), chat_id, format!("authorize_wait_for_id"), None).await?;
        telegram::send_message(chat_id, format!("What's the ID?")).await?;
    } else {
        telegram::send_message(chat_id, format!("You're not authorized to make changes")).await?;
    }
    Ok(())
}
