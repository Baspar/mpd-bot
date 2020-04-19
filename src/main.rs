use tokio::time::{delay_for, Duration};
use tokio::fs::File;
use tokio::io;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

mod structs;
use structs::{Update,Res};

pub type BoxError = std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>;

fn make_url(method: String) -> String {
    let key = std::env::var("MPD_BOT_API_KEY").unwrap();
    format!("https://api.telegram.org/bot{}/{}", key, method)
}

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

fn process_update(update: Update) -> Result<(), BoxError> {
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
    loop {
        let method = match last_update_id {
            None => String::from("getUpdates"),
            Some(id) => format!("getUpdates?offset={}", id + 1)

        };
        let res: Res<Vec<Update>> = reqwest::get(&make_url(method))
            .await?
            .json()
            .await?;
        println!("{} updates", res.result.len());
        for update in res.result {
            let update_id = update.update_id;
            last_update_id = Some(update_id);
            match process_update(update) {
                Err(err) => println!("[{}] {}", update_id, err),
                _ => {}
            }
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
