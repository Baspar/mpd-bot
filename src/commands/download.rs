use std::sync::Mutex;
use std::sync::Arc;
use rusqlite::Connection;
use tokio::fs::File;
use tokio::io;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use crate::db;
use crate::telegram;
use crate::utils::{BoxError,read_entity_from_text};
use crate::telegram::structs::MessageEntity;

async fn download_file(url: String, filename: String) -> Result<(), BoxError> {
    println!("Downloading {}", url);
    let mut response = reqwest::get(&url).await?
        .error_for_status()?
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = File::create(filename).await?;
    io::copy(&mut response, &mut file).await?;
    println!("{} downloaded", url);
    Ok(())
}

pub async fn download(conn: Arc<Mutex<Connection>>, chat_id: i64, entities: Vec<MessageEntity>, text: String) -> Result<(), BoxError> {
    let url_entities = entities
        .iter()
        .filter(|entity| entity.t == "url")
        .map(|entity| read_entity_from_text(entity, text.clone()))
        .next();
    if let Some(url) = url_entities {
        db::set_chat_status(conn.clone(), chat_id, "download_wait_for_filename".to_string(), Some(url)).await?;
        telegram::send_message(chat_id, "What's the filename ?".to_string()).await?;
    } else {
        db::set_chat_status(conn.clone(), chat_id, "download_wait_for_url".to_string(), None).await?;
        telegram::send_message(chat_id, "Give me the URL".to_string()).await?;
    }
    Ok(())
}

pub async fn filename(conn: Arc<Mutex<Connection>>, chat_id: i64, filename: String, url: String) -> Result<(), BoxError> {
    telegram::send_message(chat_id, format!("Downloading {}", filename)).await?;
    db::set_chat_status(conn, chat_id, "download_wait_for_command".to_string(), None).await?;
    let message = match download_file(url, filename.clone()).await {
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
            db::set_chat_status(conn.clone(), chat_id, "download_wait_for_filename".to_string(), Some(url)).await?;
            telegram::send_message(chat_id, "What's the filename ?".to_string()).await?;
            return Ok(())
        }
    }
    telegram::send_message(chat_id, "I can't recognize any URL, please try again or /cancel".to_string()).await?;
    Ok(())
}
