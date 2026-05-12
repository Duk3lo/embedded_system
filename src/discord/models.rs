use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DiscordMessage {
    pub id: String,
    pub content: String,
}