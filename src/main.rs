use std::sync::Mutex;
use tokio::time::{delay_for, Duration};
use std::sync::Arc;
use rusqlite::Connection;

mod telegram;
use telegram::structs::{Update,Message};

mod db;

mod utils;
use utils::{BoxError,CustomError,read_entity_from_text};

mod commands;
use commands::{download,cancel,authorize};

mod action;

async fn process_wait_for_command(conn: Arc<Mutex<Connection>>, message: Message) -> Result<(), BoxError> {
    let text = message.text.ok_or("no text found")?;
    let chat_id = message.chat.id;
    let entities = message.entities;
    if let Some(entities) = entities {
        let command = entities
            .iter()
            .find(|entity| entity.t == "bot_command")
            .map(|entity| read_entity_from_text(entity, text.clone()))
            .ok_or("Waiting for a command")?;

        println!("[{}] {}", chat_id, command);
        match command.as_str() {
            "/authorize" => authorize::authorize(conn, chat_id, text).await?,
            "/download" => download::download(conn, chat_id, entities, text).await?,
            _ => telegram::send_message(chat_id, format!("Command {} doesn't exist", command)).await?
        }
    } else {
        telegram::send_message(chat_id, format!("I don't get what you mean")).await?;
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

    let text = message.text.clone().ok_or("no text found")?;
    let entities = message.entities.clone();
    if let Some(entities) = entities.clone() {
        let is_cancel = entities
            .iter()
            .filter(|entity| entity.t == "bot_command")
            .map(|entity| read_entity_from_text(entity, text.clone()))
            .find(|command| command == "/cancel");
        if is_cancel.is_some() {
            cancel::cancel(conn, chat_id).await?;
            return Ok(())
        }
    }

    let (status, params) = db::get_chat_status(conn.clone(), chat_id).await?;
    match status.as_str() {
        "wait_for_command" => process_wait_for_command(conn, message).await?,
        "wait_for_url" => download::url(conn, chat_id, entities, text).await?,
        "wait_for_filename" => download::filename(conn, chat_id, text, params).await?,
        _ => telegram::send_message(chat_id, format!("I don't get what you mean")).await?
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let mut last_update_id: Option<i64> = None;
    let conn = tokio::task::spawn_blocking(db::init).await??;
    loop {
        match telegram::get_update(&last_update_id).await {
            Ok(res) => {
                println!("{} updates", res.result.len());
                for update in res.result {
                    let update_id = update.update_id;
                    last_update_id = Some(update_id);
                    match process_update(conn.clone(), update).await {
                        Err(err) => println!("[{}] {}", update_id, err),
                        _ => {}
                    }
                }
            },
            Err(e) => println!("Error: {}", e)
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
