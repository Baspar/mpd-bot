pub mod structs;
use crate::utils::BoxError;
use structs::{Res, Update};

fn make_url(method: String) -> Result<String, BoxError> {
    let key = std::env::var("MPD_BOT_API_KEY")?;
    Ok(format!("https://api.telegram.org/bot{}/{}", key, method))
}

pub async fn get_update(last_update_id: &Option<i64>) -> Result<Res<Vec<Update>>, BoxError> {
    let method = match last_update_id {
        None => String::from("getUpdates"),
        Some(id) => format!("getUpdates?offset={}", id + 1)
    };
    let res = reqwest::get(&make_url(method)?)
        .await?
        .json()
        .await?;
    Ok(res)
}
