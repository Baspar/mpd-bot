use serde::Deserialize;

#[derive(Deserialize,Debug)]
pub struct GetMe {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: String,
    pub language_code: Option<String>,
    pub can_join_groups: bool,
    pub can_read_all_group_messages: bool,
    pub supports_inline_queries: bool
}
#[derive(Deserialize,Debug)]
pub struct Chat {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>
}
#[derive(Deserialize,Debug)]
pub struct Message {
    pub text: Option<String>,
    pub chat: Chat
}
#[derive(Deserialize,Debug)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>
}
#[derive(Deserialize,Debug)]
pub struct Res<T> {
    pub ok: bool,
    pub result: T
}

