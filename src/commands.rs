use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::db;
use crate::telegram;
use crate::utils::{BoxError,read_entity_from_text};
use crate::telegram::structs::MessageEntity;
use crate::action;

pub async fn cancel(conn: Arc<Mutex<Connection>>, chat_id: i64) -> Result<(), BoxError> {
    db::set_chat_status(conn, chat_id, format!("wait_for_command"), None).await?;
    telegram::send_message(chat_id, format!("Task cancelled")).await?;
    Ok(())
}

pub async fn download(conn: Arc<Mutex<Connection>>, chat_id: i64, entities: Vec<MessageEntity>, text: String) -> Result<(), BoxError> {
    let url_entities = entities
        .iter()
        .filter(|entity| entity.t == "url")
        .map(|entity| read_entity_from_text(entity, text.clone()))
        .next();
    if let Some(url) = url_entities {
        db::set_chat_status(conn.clone(), chat_id, format!("wait_for_filename"), Some(url)).await?;
        telegram::send_message(chat_id, format!("What's the filename ?")).await?;
    } else {
        db::set_chat_status(conn.clone(), chat_id, format!("wait_for_url"), None).await?;
        telegram::send_message(chat_id, format!("Give me the URL")).await?;
    }
    Ok(())
}

pub async fn filename(conn: Arc<Mutex<Connection>>, chat_id: i64, filename: String, url: String) -> Result<(), BoxError> {
    telegram::send_message(chat_id, format!("Downloading {}", filename)).await?;
    db::set_chat_status(conn, chat_id, format!("wait_for_command"), None).await?;
    let message = match action::download_file(url, filename.clone()).await {
        Ok(_) => format!("{} downloaded", filename),
        Err(err) => format!("cannot download {}", err)
    };
    telegram::send_message(chat_id, message).await?;
    Ok(())
}

pub async fn url(conn: Arc<Mutex<Connection>>, chat_id: i64, entities: Option<Vec<MessageEntity>>, text: String) -> Result<(), BoxError> {
    if let Some(entities) = entities {
        let url = entities
            .iter()
            .find(|entity| entity.t == "url")
            .map(|entity| read_entity_from_text(entity, text.clone()));
        if let Some(url) = url {
            db::set_chat_status(conn.clone(), chat_id, format!("wait_for_filename"), Some(url)).await?;
            telegram::send_message(chat_id, format!("What's the filename ?")).await?;
        } else {
            telegram::send_message(chat_id, format!("I can't recognize any URL, please try again or /cancel")).await?;
        }
    } else {
        telegram::send_message(chat_id, format!("I can't recognize any URL, please try again or /cancel")).await?;
    }
    Ok(())
}
    Ok(())
}
