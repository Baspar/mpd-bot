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
    let response = reqwest::get(&url).await?;
    let response = response.error_for_status()?;
    let response = response.bytes_stream();
    let response = response.map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e));
    let response = response.into_async_read();
    let mut response = response.compat();

    let mut file = File::create("music.audio").await?;
    io::copy(&mut response, &mut file).await?;
    println!("{} downloaded", url);
    Ok(())
}

fn process_update(update: Update) {
    if let Some(message) = update.message {
        match (message.text, message.entities) {
            (Some(text), Some(entities)) => {
                let url_entities = entities
                    .iter()
                    .filter(|entity| entity.t == "url")
                    .map(|entity| String::from(text.get(entity.offset..entity.offset + entity.length).unwrap()));
                for url_entity in url_entities {
                    tokio::spawn(download_file(url_entity));
                }
            },
            _ => {}
        }
    }
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
            last_update_id = Some(update.update_id);
            process_update(update);
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
