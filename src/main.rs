use tokio::time::{delay_for, Duration};
// use tokio::task::spawn_blocking;

mod structs;
use structs::{Update,Res};

fn make_url(method: String) -> String {
    let key = std::env::var("MPD_BOT_API_KEY").unwrap();
    format!("https://api.telegram.org/bot{}/{}", key, method)
}

async fn process_update(update: Update) {
    if let Some(message) = update.message {
        match (message.text, message.entities) {
            (Some(text), Some(entities)) => {
                let url_entities = entities
                    .iter()
                    .filter(|entity| entity.t == "url")
                    .map(|entity| String::from(text.get(entity.offset..entity.offset + entity.length).unwrap()));
                for url_entity in url_entities {
                    println!("{}", url_entity);
                }
            },
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            tokio::spawn(process_update(update));
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
