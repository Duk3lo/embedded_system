pub fn start_discord_task(token: String, channel: String, app_id: String) {
    std::thread::Builder::new()
        .name("discord_task".into())
        .stack_size(20 * 1024)
        .spawn(move || {
            // Esperamos internet síncronamente
            crate::discord::bot::wait_for_internet();
            // Ejecutamos el bot (que registrará comandos y abrirá WS)
            crate::discord::bot::run_bot(token, channel, app_id);
        })
        .unwrap();
}