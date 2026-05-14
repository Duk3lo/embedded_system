use crate::discord::send::send_reply;
use log::info;

pub struct BotCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub action: fn(token: &str, channel: &str),
}

pub const COMMANDS: &[BotCommand] = &[
    BotCommand {
        name: "!ping",
        description: "Responde con un Pong y verifica la conexión.",
        action: |t, c| {
            send_reply(t, c, r#"{"content":"🏓 ¡Pong desde el ESP32!"}"#);
        },
    },
    BotCommand {
        name: "!status",
        description: "Muestra el estado de la memoria RAM del ESP32.",
        action: |t, c| {
            let ram = unsafe { esp_idf_sys::esp_get_free_heap_size() };
            let payload = format!(r#"{{"content":"📊 RAM Libre: {} bytes"}}"#, ram);
            send_reply(t, c, &payload);
        },
    },
    BotCommand {
        name: "!help",
        description: "Muestra esta lista de comandos.",
        action: |t, c| {
            let mut help_msg = String::from("--- 🤖 Comandos Disponibles ---\\n");
            for cmd in COMMANDS {
                help_msg.push_str(&format!("**{}**: {}\\n", cmd.name, cmd.description));
            }
            let payload = format!(r#"{{"content":"{}"}}"#, help_msg);
            send_reply(t, c, &payload);
        },
    },
];

pub fn handle_command(content: &str, token: &str, channel_id: &str) {
    let content = content.trim().to_lowercase();
    
    if let Some(cmd) = COMMANDS.iter().find(|c| content == c.name) {
        info!("Ejecutando comando: {}", cmd.name);
        (cmd.action)(token, channel_id);
    }
}