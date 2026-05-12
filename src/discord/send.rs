use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
use esp_idf_svc::io::{Write};
use log::error;

pub fn send_reply(discord_token: &str, channel_id: &str, payload: &str) {
    let connection = match EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size_tx: Some(2048),
        ..Default::default()
    }) {
        Ok(c) => c,
        Err(e) => { error!("Error POST connection: {:?}", e); return; }
    };

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    let auth = format!("Bot {}", discord_token);

    let headers = [
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
        ("Connection", "close"),
    ];

    if let Ok(mut request) = client.request(Method::Post, &url, &headers) {
        let _ = request.write_all(payload.as_bytes());
        if let Ok(mut response) = request.submit() {
            // Importante: Drenar la respuesta para cerrar el socket limpiamente
            let mut dummy = [0u8; 128];
            while let Ok(s) = response.read(&mut dummy) { if s == 0 { break; } }
        }
    }
}