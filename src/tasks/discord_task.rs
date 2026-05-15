use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::wifi::wifi::WifiCommand;

pub fn start_discord_task(
    token: String, 
    app_id: String, 
    wifi_tx: Sender<WifiCommand>, 
    bot_active: Arc<AtomicBool>
) {
    std::thread::Builder::new()
        .name("discord_task".into())
        .stack_size(20 * 1024)
        .spawn(move || {
            crate::discord::bot::run_bot(token, app_id, wifi_tx, bot_active);
        })
        .unwrap();
}