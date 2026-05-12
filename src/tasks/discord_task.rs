use crate::discord::bot::run_bot;

use std::thread;

pub fn start_discord_task(
    discord_token: String,
    channel_id: String,
) {

    thread::Builder::new()
        .name("discord_task".into())
        .stack_size(20 * 1024)
        .spawn(move || {

            run_bot(
                discord_token,
                channel_id
            );

        })
        .unwrap();
}