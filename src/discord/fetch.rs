use crate::discord::models::DiscordMessage;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;

struct EspToStdReader<R>(R);
impl<R: esp_idf_svc::io::Read> std::io::Read for EspToStdReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e:?}")))
    }
}

pub fn fetch_last_message(discord_token: &str, channel_id: &str) -> Option<DiscordMessage> {
    let connection = EspHttpConnection::new(&HttpConfig {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    }).ok()?;

    let mut client = Client::wrap(connection);
    let url = format!("https://discord.com/api/v10/channels/{}/messages?limit=1", channel_id);
    let auth = format!("Bot {}", discord_token);

    let headers = [("Authorization", auth.as_str()), ("User-Agent", "ESP32")];

    if let Ok(request) = client.request(Method::Get, &url, &headers) {
        if let Ok(response) = request.submit() {
            if response.status() == 200 {
                let reader = EspToStdReader(response);
                let msgs: Result<Vec<DiscordMessage>, _> = serde_json::from_reader(reader);
                return msgs.ok()?.into_iter().next();
            }
        }
    }
    None
}