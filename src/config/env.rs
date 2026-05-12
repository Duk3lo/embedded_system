pub struct AppConfig {
    pub wifi_ssid: String,
    pub wifi_pass: String,
    pub discord_token: String,
    pub channel_id: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Usamos env! para grabarlas en el binario durante la compilación
        Ok(Self {
            wifi_ssid: env!("WIFI_SSID").to_string(),
            wifi_pass: env!("WIFI_PASS").to_string(),
            discord_token: env!("DISCORD_TOKEN").to_string(),
            channel_id: env!("CHANNEL_ID").to_string(),
        })
    }
}