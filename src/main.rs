use tokio::sync::Mutex;
use tokio::time::{delay_for, Duration};
use tokio::fs::File;
use tokio::io;
use std::sync::Arc;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use rusqlite::Connection;

mod telegram;
use telegram::structs::Update;

mod db;

mod utils;
use utils::BoxError;

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

fn process_update(_conn: Arc<Mutex<Connection>>, update: Update) -> Result<(), BoxError> {
    let message = update.message.ok_or("No message found")?;
    let text = message.text.ok_or("no text found")?;
    let entities = message.entities.ok_or("no entities found")?;
    let url_entities = entities
        .iter()
        .filter(|entity| entity.t == "url")
        .map(|entity| String::from(text.get(entity.offset..entity.offset + entity.length).unwrap()));
    for url_entity in url_entities {
        tokio::spawn(download_file(url_entity));
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
            match process_update(conn.clone(), update) {
                Err(err) => println!("[{}] {}", update_id, err),
                _ => {}
            }
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
