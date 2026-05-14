pub fn get_response_for(command: &str) -> String {
    let cmd = command.strip_prefix('!').unwrap_or(command).trim().to_lowercase();
    match cmd.as_str() {
        "ping" => "🏓 ¡Pong desde el ESP32!".to_string(),
        "status" => {
            let ram = unsafe { esp_idf_sys::esp_get_free_heap_size() };
            format!("📊 RAM Libre: {} bytes", ram)
        },
        _ => "Comando no reconocido".to_string(),
    }
}