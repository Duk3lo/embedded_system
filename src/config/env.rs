pub struct AppConfig {
    pub wifi_ssid: String,
    pub wifi_pass: String,
    pub discord_token: String,
    pub channel_id: String,
    pub discord_app_id: String, // <-- Nueva variable
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            wifi_ssid: env!("WIFI_SSID").to_string(),
            wifi_pass: env!("WIFI_PASS").to_string(),
            discord_token: env!("DISCORD_TOKEN").to_string(),
            channel_id: env!("CHANNEL_ID").to_string(),
            discord_app_id: env!("DISCORD_APP_ID").to_string(), // <-- Nueva carga
        })
    }
}