use crate::discord::models::DiscordMessage;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
use log::{warn};

pub fn fetch_last_message(discord_token: &str, channel_id: &str) -> Option<DiscordMessage> {
    let connection = EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size: Some(4096),
        ..Default::default()
    }).ok()?;

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/channels/{}/messages?limit=1", channel_id);
    let auth = format!("Bot {}", discord_token);

    let headers = [
        ("Authorization", auth.as_str()),
        ("Accept", "application/json"),
        ("User-Agent", "ESP32-Rust-Bot"),
        ("Connection", "close"),
    ];

    if let Ok(request) = client.request(Method::Get, &url, &headers) {
        if let Ok(mut response) = request.submit() {
            if response.status() == 200 {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 1024];
                while let Ok(size) = response.read(&mut chunk) {
                    if size == 0 { break; }
                    buf.extend_from_slice(&chunk[..size]);
                }
                return serde_json::from_slice::<Vec<DiscordMessage>>(&buf)
                    .ok()?
                    .into_iter()
                    .next();
            } else {
                warn!("Error Discord GET: {}", response.status());
            }
        }
    }
    None
}