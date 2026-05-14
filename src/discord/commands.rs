use crate::discord::send::send_reply;
use crate::discord::logic::get_response_for;
use log::info;

pub fn handle_command(content: &str, token: &str, channel_id: &str) {
    let content = content.trim();
    if content.starts_with('!') {
        info!("Ejecutando comando de texto: {}", content);
        let response_text = get_response_for(content);
        let payload = serde_json::json!({ "content": response_text }).to_string();
        send_reply(token, channel_id, &payload);
    }
}