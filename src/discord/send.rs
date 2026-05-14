use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
// Importamos Read para poder drenar la respuesta
use esp_idf_svc::io::{Write}; 
use log::{error, info};

pub fn send_reply(discord_token: &str, channel_id: &str, payload: &str) {
    let connection = match EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size_tx: Some(1024), 
        ..Default::default()
    }) {
        Ok(c) => c,
        Err(e) => { error!("Error de conexión: {:?}", e); return; }
    };

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    let auth = format!("Bot {}", discord_token);

    let headers = [
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
        ("Connection", "close"), // Crucial para liberar RAM rápido
    ];

    if let Ok(mut request) = client.request(Method::Post, &url, &headers) {
        let _ = request.write_all(payload.as_bytes());
        
        if let Ok(mut response) = request.submit() {
            let status = response.status();
            
            // Drenar la respuesta: leemos todo el cuerpo pero lo ignoramos.
            // Esto le indica al stack TCP que ya terminamos y puede liberar la RAM.
            let mut dummy_buf = [0u8; 128];
            while let Ok(n) = response.read(&mut dummy_buf) {
                if n == 0 { break; }
            }

            if status != 200 && status != 201 {
                error!("Discord respondió con error: {}", status);
            } else {
                info!("Respuesta enviada con éxito (Status: {})", status);
            }
        }
    }
}