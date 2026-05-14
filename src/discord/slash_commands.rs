use crate::discord::logic::get_response_for;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
use esp_idf_svc::io::Write;
use log::{error, info};
use serde_json::json;

pub fn register_slash_commands(discord_token: &str, app_id: &str) {
    info!("🚀 Sincronizando comandos Slash para App ID: {}...", app_id);

    let url = format!("https://discord.com/api/v10/applications/{}/commands", app_id);
    let auth = format!("Bot {}", discord_token);
    
    let commands_json = json!([
        {
            "name": "ping",
            "description": "Verifica si el ESP32 esta vivo",
            "type": 1,
            "integration_types": [0, 1],
            "contexts": [0, 1, 2]
        },
        {
            "name": "status",
            "description": "Muestra la RAM libre del ESP32",
            "type": 1,
            "integration_types": [0, 1],
            "contexts": [0, 1, 2]
        }
    ]);

    let payload = commands_json.to_string();
    let content_length = payload.len().to_string();

    let headers = [
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
        ("Content-Length", content_length.as_str()),
        ("User-Agent", "ESP32-Rust"),
        ("Connection", "close"),
    ];

    if let Ok(conn) = EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size_tx: Some(2048),
        ..Default::default()
    }) {
        let mut client = Client::wrap(conn);
        if let Ok(mut request) = client.request(Method::Put, &url, &headers) {
            let _ = request.write_all(payload.as_bytes());
            let _ = request.submit();
            info!("✅ Registro de comandos finalizado.");
        }
    }
}

pub fn handle_interaction(command_name: &str, interaction_id: &str, interaction_token: &str) {
    info!("Ejecutando Slash Command: /{}", command_name);
    let response_text = get_response_for(command_name);
    send_interaction_reply(interaction_id, interaction_token, &response_text);
}

fn send_interaction_reply(interaction_id: &str, interaction_token: &str, content: &str) {
    let payload_json = serde_json::json!({
        "type": 4,
        "data": { "content": content }
    });
    let payload = payload_json.to_string();
    let content_length = payload.len().to_string();

    let connection = match EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size_tx: Some(2048),
        ..Default::default()
    }) {
        Ok(c) => c,
        Err(e) => { error!("Error HTTP: {:?}", e); return; }
    };

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/interactions/{}/{}/callback", interaction_id, interaction_token);
    
    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", content_length.as_str()),
        ("Connection", "close"),
    ];

    if let Ok(mut request) = client.request(Method::Post, &url, &headers) {
        let _ = request.write_all(payload.as_bytes());
        if let Ok(_) = request.submit() {
            info!("✅ Respuesta Slash enviada.");
        }
    }
}
