use crate::discord::fetch::fetch_last_message;
use crate::discord::commands::handle_command;
use log::info;

pub fn run_bot(discord_token: String, channel_id: String) {
    let mut last_message_id = String::new();
    
    info!("Bot iniciado. Escuchando canal principal: {}", channel_id);

    loop {
        // En un ESP32, no podemos "escuchar" todo a la vez por HTTP.
        // Consultamos el canal configurado. 
        if let Some(msg) = fetch_last_message(&discord_token, &channel_id) {
            if msg.id != last_message_id {
                last_message_id = msg.id.clone();
                info!("Mensaje recibido: {}", msg.content);

                // Si el mensaje empieza con / o es una mención, procesamos
                handle_command(&msg.content, &discord_token, &channel_id);
            }
        }
        // Pausa necesaria para no ser bloqueado por Discord
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}