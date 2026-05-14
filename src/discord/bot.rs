use crate::discord::fetch::fetch_last_message;
use crate::discord::commands::handle_command;
use log::info;

pub fn run_bot(discord_token: String, channel_id: String) {
    let mut last_message_id = String::new();
    
    info!("Bot iniciado. Escuchando canal principal: {}", channel_id);

    loop {
        if let Some(msg) = fetch_last_message(&discord_token, &channel_id) {
            if msg.id != last_message_id {
                last_message_id = msg.id.clone();
                info!("Mensaje recibido: {}", msg.content);

                handle_command(&msg.content, &discord_token, &channel_id);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}