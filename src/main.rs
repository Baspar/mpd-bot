use tokio::time::{delay_for, Duration};
use tokio::task::spawn_blocking;

mod structs;
use structs::{Update,Res};

fn make_url(method: String) -> String {
    let key = std::env::var("MPD_BOT_API_KEY").unwrap();
    format!("https://api.telegram.org/bot{}/{}", key, method)
}

async fn process_update(update: &Update) {
    println!("{:?}", update);
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
        for update in &res.result {
            last_update_id = Some(update.update_id);
            process_update(update).await;
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
