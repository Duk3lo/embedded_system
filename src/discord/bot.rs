use crate::discord::fetch::fetch_last_message;
use crate::discord::send::send_reply;

use log::info;

pub fn run_bot(
    discord_token: String,
    channel_id: String,
) {

    let mut last_message_id = String::new();

    loop {

        if let Some(msg) =
            fetch_last_message(
                &discord_token,
                &channel_id
            ) {

            if msg.id != last_message_id {

                info!("Mensaje: {}", msg.content);

                last_message_id = msg.id.clone();

                if msg.content.trim() == "!ping" {

                    let payload =
                        r#"{"content":"¡Pong desde ESP32!"}"#;

                    send_reply(
                        &discord_token,
                        &channel_id,
                        payload
                    );
                }
            }
        }

        std::thread::sleep(
            std::time::Duration::from_secs(3)
        );
    }
}