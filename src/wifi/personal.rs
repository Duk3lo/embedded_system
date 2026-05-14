use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi};
use log::{info, warn};
use std::thread::sleep;
use std::time::Duration;

pub fn connect_wifi(wifi: &mut EspWifi, ssid: &str, pass: &str) -> anyhow::Result<()> {
    let wifi_config = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into()?,
        password: pass.try_into()?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;
    wifi.start()?;

    const MAX_RETRIES: u32 = 0;
    let mut attempt: u32 = 0;

    loop {
        attempt += 1;
        info!("Intentando conectar a {}... (intento {})", ssid, attempt);

        if let Err(e) = wifi.connect() {
            warn!("connect() falló: {e}");
        }

        for _ in 0..10 {
            if wifi.is_connected().unwrap_or(false) {
                info!("¡Wi-Fi conectado!");
                return Ok(());
            }
            sleep(Duration::from_secs(1));
        }

        warn!("Fallo de conexión. Reintentando...");

        let _ = wifi.disconnect();
        sleep(Duration::from_secs(2));

        if MAX_RETRIES != 0 && attempt >= MAX_RETRIES {
            anyhow::bail!("No se pudo conectar a Wi-Fi tras {attempt} intentos");
        }
    }
}