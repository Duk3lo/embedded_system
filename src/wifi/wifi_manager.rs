use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi};
use log::{info, warn};
use std::thread::sleep;
use std::time::Duration;

pub fn connect_wifi(wifi: &mut EspWifi, ssid: &str, pass: &str) -> anyhow::Result<()> {
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: pass.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;

    loop {
        info!("Intentando conectar a {}...", ssid);
        let _ = wifi.connect();

        for _ in 0..10 {
            if wifi.is_connected().unwrap_or(false) {
                info!("¡Wi-Fi Conectado!");
                return Ok(());
            }
            sleep(Duration::from_secs(1));
        }

        warn!("Fallo de conexión. Reintentando...");
        let _ = wifi.disconnect();
        sleep(Duration::from_secs(2));
    }
}