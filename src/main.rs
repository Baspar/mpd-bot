use std::sync::Mutex;
use tokio::time::{delay_for, Duration};
use tokio::fs::File;
use tokio::io;
use std::sync::Arc;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use rusqlite::Connection;

mod telegram;
use telegram::structs::{Update,Message};

mod db;

mod utils;
use utils::{BoxError,CustomError,read_entity_from_text};

mod commands;

async fn download_file(url: String) -> Result<(), BoxError> {
    println!("Downloading {}", url);
    let mut response = reqwest::get(&url).await?
        .error_for_status()?
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = File::create("music.audio").await?;
    io::copy(&mut response, &mut file).await?;
    println!("{} downloaded", url);
    Ok(())
}

async fn process_wait_for_command(conn: Arc<Mutex<Connection>>, message: Message) -> Result<(), BoxError> {
    let text = message.text.clone().ok_or("no text found")?;
    let entities = message.entities.clone().ok_or("no entities found")?;
    let chat_id = message.chat.id;
    let command = entities
        .iter()
        .find(|entity| entity.t == "bot_command")
        .map(|entity| read_entity_from_text(entity, text.clone()))
        .ok_or("Waiting for a command")?;

    println!("[{}] {}", chat_id, command);
    match command.as_str() {
        "/cancel" => commands::cancel(conn, chat_id).await?,
        "/download" => commands::download(conn, chat_id, entities, text).await?,
        _ => telegram::send_message(chat_id, format!("Command {} doesn't exist", command)).await?
    }
    Ok(())
}

async fn process_update(conn: Arc<Mutex<Connection>>, update: Update) -> Result<(), BoxError> {
    let message = update.message.clone().ok_or("No message found")?;
    let chat_id = message.chat.id;
    if !db::is_chat_authorized(conn.clone(), chat_id).await? {
        telegram::send_message(chat_id, format!("Your chat is not authorized (#{})", chat_id)).await?;
        return Err(Box::new(CustomError::new(format!("Chat {} not authorized", chat_id))))
    }

    let status = db::get_chat_status(conn.clone(), chat_id).await?;

    match status.as_str() {
        "wait_for_command" => process_wait_for_command(conn, message).await?,
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let mut last_update_id: Option<i64> = None;
    let conn = tokio::task::spawn_blocking(db::init).await??;
    loop {
        let res = telegram::get_update(&last_update_id).await?;
        println!("{} updates", res.result.len());
        for update in res.result {
            let update_id = update.update_id;
            last_update_id = Some(update_id);
            match process_update(conn.clone(), update).await {
                Err(err) => println!("[{}] {}", update_id, err),
                _ => {}
            }
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
