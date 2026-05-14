use crate::discord::commands::handle_command;
use crate::discord::slash_commands::{handle_interaction, register_slash_commands};
use log::{error, info, warn};
use serde_json::json;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::time::Duration;

use esp_idf_svc::ws::client::{EspWebSocketClient, EspWebSocketClientConfig, WebSocketEventType};
use esp_idf_svc::ws::FrameType;

enum BotEvent {
    Op10Hello,
    Message(String, String, String),
    Interaction(String, String, String),
}

pub fn wait_for_internet() {
    info!("🔍 Verificando conexión real a Internet (DNS)...");
    loop {
        if "discord.com:443".to_socket_addrs().is_ok() {
            info!("✅ ¡Internet listo y verificado!");
            break;
        }
        warn!("⏳ Esperando a que el DNS responda...");
        std::thread::sleep(Duration::from_secs(2));
    }
}

pub fn run_bot(token: String, default_channel_id: String, app_id: String) {
    info!("🤖 Iniciando ciclo de vida del Bot...");

    loop {
        info!("🔍 Verificando conexión a Internet...");
        if "discord.com:443".to_socket_addrs().is_ok() {
            info!("✅ Internet detectado.");
            break;
        }
        warn!("⏳ Esperando DNS... El router aún no provee Internet.");
        std::thread::sleep(Duration::from_secs(3));
    }

    register_slash_commands(&token, &app_id);
    let connection_config = EspWebSocketClientConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size: 10240,
        task_stack: 8192,
        network_timeout_ms: Duration::from_millis(10000),
        reconnect_timeout_ms: Duration::from_millis(10000),
        ..Default::default()
    };

    let token_ptr = token.clone();
    let channel_ptr = default_channel_id.clone();
    let (tx, rx) = mpsc::channel::<BotEvent>();
    let mut json_buffer = String::new();

    let mut client = EspWebSocketClient::new(
        "wss://gateway.discord.gg/?v=10&encoding=json",
        &connection_config,
        Duration::from_secs(10),
        move |event_result| match event_result {
            Ok(ws_event) => match ws_event.event_type {
                WebSocketEventType::Text(text) => {
                    json_buffer.push_str(text);
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_buffer) {
                        process_discord_event(v, &token_ptr, &channel_ptr, &tx);
                        json_buffer.clear();
                    } else if json_buffer.len() > 32768 {
                        json_buffer.clear();
                    }
                }
                WebSocketEventType::Connected => info!("🔌 WebSocket Conectado!"),
                WebSocketEventType::Disconnected => error!("❌ WebSocket Desconectado"),
                _ => {}
            },
            Err(e) => error!("⚠️ Error WS: {:?}", e),
        },
    )
    .expect("No se pudo crear el cliente WS");

    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(BotEvent::Op10Hello) => {
                info!("Enviando IDENTIFY...");
                let identify = json!({
                    "op": 2,
                    "d": {
                        "token": token,
                        "intents": 37377,
                        "properties": { "os": "esp32", "browser": "esp-bot", "device": "esp32" }
                    }
                })
                .to_string();
                let _ = client.send(FrameType::Text(false), identify.as_bytes());
            }
            Ok(BotEvent::Message(content, t, c)) => handle_command(&content, &t, &c),
            Ok(BotEvent::Interaction(cmd, id, tok)) => {
                std::thread::spawn(move || {
                    handle_interaction(&cmd, &id, &tok);
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let hb = json!({"op": 1, "d": null}).to_string();
                let _ = client.send(FrameType::Text(false), hb.as_bytes());
                info!("💓 Heartbeat.");
            }
            Err(_) => break,
        }
    }
}

fn process_discord_event(
    v: serde_json::Value,
    token: &str,
    def_channel: &str,
    tx: &mpsc::Sender<BotEvent>,
) {
    match v["op"].as_u64() {
        Some(10) => {
            let _ = tx.send(BotEvent::Op10Hello);
        }
        Some(0) => {
            let t = v["t"].as_str().unwrap_or("");
            let d = &v["d"];

            if t == "MESSAGE_CREATE" && d["author"]["bot"].as_bool() != Some(true) {
                let _ = tx.send(BotEvent::Message(
                    d["content"].as_str().unwrap_or("").to_string(),
                    token.to_string(),
                    d["channel_id"].as_str().unwrap_or(def_channel).to_string(),
                ));
            } else if t == "INTERACTION_CREATE" && d["type"].as_u64() == Some(2) {
                let _ = tx.send(BotEvent::Interaction(
                    d["data"]["name"].as_str().unwrap_or("").to_string(),
                    d["id"].as_str().unwrap_or("").to_string(),
                    d["token"].as_str().unwrap_or("").to_string(),
                ));
            }
        }
        _ => {}
    }
}
