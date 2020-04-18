use tokio::time::{delay_for, Duration};
use tokio::task::spawn_blocking;

mod structs;
use structs::{Update,Res};

fn make_url(method: &str) -> String {
    let key = std::env::var("MPD_BOT_API_KEY").unwrap();
    format!("https://api.telegram.org/bot{}/{}", key, method)
}

async fn process_update(update: &Update) {
    println!("{:?}", update);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let res: Res<Vec<Update>> = reqwest::get(&make_url("getUpdates"))
            .await?
            .json()
            .await?;
        println!("{} updates", res.result.len());
        for update in &res.result {
            process_update(update).await;
        }
        delay_for(Duration::from_secs(1)).await;
    }
}
