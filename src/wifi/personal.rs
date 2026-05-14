use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi};
use log::{info};
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
    
    // El driver interno de ESP-IDF intentará conectar automáticamente al iniciar.
    // Solo llamamos a connect() si no lo está intentando ya.
    info!("Iniciando proceso de conexión a {}...", ssid);
    let _ = wifi.connect(); 

    // Aumentamos el tiempo de espera a 20 segundos para dar margen al router y al DHCP
    for segundo in 1..=20 {
        sleep(Duration::from_secs(1));
        
        if wifi.is_connected().unwrap_or(false) {
            info!("¡Wi-Fi conectado a nivel de radio en el segundo {}!", segundo);
            // Espera extra vital para que el router asigne la IP (DHCP)
            sleep(Duration::from_secs(2)); 
            return Ok(());
        }
        
        if segundo % 5 == 0 {
            info!("... todavía intentando conectar ({}s) ...", segundo);
            // Re-lanzamos la orden de conectar por si el driver se rindió
            let _ = wifi.connect();
        }
    }

    anyhow::bail!("No se pudo conectar al Wi-Fi tras 20 segundos. Revisa la señal o la clave.");
}