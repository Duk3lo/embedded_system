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
    
    // JSON Moderno para que aparezca como "App" (User Installable)
    let commands_json = json!([
        {
            "name": "ping",
            "description": "Verifica si el ESP32 esta vivo",
            "type": 1,
            "integration_types": [0, 1], // Instalable en Servidor y Usuario
            "contexts": [0, 1, 2]       // Disponible en Servidor, Bot DM y Grupos
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

// 2. RESPONDER A LAS INTERACCIONES (Cuando alguien ejecuta /ping)
pub fn handle_interaction(command_name: &str, interaction_id: &str, interaction_token: &str) {
    info!("Ejecutando Slash Command: /{}", command_name);

    // Usamos la lógica compartida
    let response_text = get_response_for(command_name);

    // Enviamos la respuesta usando el método de interacciones
    send_interaction_reply(interaction_id, interaction_token, &response_text);
}

fn send_interaction_reply(interaction_id: &str, interaction_token: &str, content: &str) {
    // 1. Preparamos el JSON antes de abrir la conexión para no perder tiempo con el socket abierto
    let payload_json = serde_json::json!({
        "type": 4,
        "data": { "content": content }
    });
    let payload = payload_json.to_string();
    let content_length = payload.len().to_string();

    // 2. Abrimos la conexión
    let connection = match EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size_tx: Some(2048), // Buffer más grande para enviar más rápido
        ..Default::default()
    }) {
        Ok(c) => c,
        Err(e) => { error!("Error HTTP: {:?}", e); return; }
    };

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/interactions/{}/{}/callback", interaction_id, interaction_token);
    
    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", content_length.as_str()), // <-- MUY IMPORTANTE
        ("Connection", "close"),
    ];

    if let Ok(mut request) = client.request(Method::Post, &url, &headers) {
        let _ = request.write_all(payload.as_bytes());
        if let Ok(_) = request.submit() {
            // No perdemos tiempo leyendo la respuesta de Discord si no es necesario
            info!("✅ Respuesta Slash enviada.");
        }
    }
}
