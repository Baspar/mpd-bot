use serde::Deserialize;
use tokio::time::{delay_for, Duration};
use tokio::task::spawn_blocking;

#[derive(Deserialize,Debug)]
struct GetMe {
    id: i64,
    is_bot: bool,
    first_name: String,
    last_name: Option<String>,
    username: String,
    language_code: Option<String>,
    can_join_groups: bool,
    can_read_all_group_messages: bool,
    supports_inline_queries: bool
}
#[derive(Deserialize,Debug)]
struct Chat {
    id: i64,
    first_name: Option<String>,
    last_name: Option<String>
}
#[derive(Deserialize,Debug)]
struct Message {
    text: Option<String>,
    chat: Chat
}
#[derive(Deserialize,Debug)]
struct Update {
    update_id: i64,
    message: Option<Message>
}
#[derive(Deserialize,Debug)]
struct Res<T> {
    ok: bool,
    result: T
}

fn make_url(method: &str) -> String {
    let key = std::env::var("MPD_BOT_API_KEY").unwrap();
    format!("https://api.telegram.org/bot{}/{}", key, method)
}

async fn process_update(update: Update) {
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
        }
        delay_for(Duration::from_secs(1)).await;
    }
    Ok(())
}
