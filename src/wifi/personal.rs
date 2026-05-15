use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi};
use log::{error, info};
use std::thread::sleep;
use std::time::Duration;

pub fn scan_wifi(wifi: &mut EspWifi) -> Vec<String> {
    info!("📡 Escaneando redes Wi-Fi...");
    match wifi.scan() {
        Ok(aps) => {
            let mut redes: Vec<String> = Vec::new();
            
            for ap in aps {
                let ssid_str = ap.ssid.to_string();
                
                if !redes.contains(&ssid_str) && !ssid_str.is_empty() {
                    redes.push(ssid_str);
                }
            }
            info!("Encontradas {} redes únicas", redes.len());
            redes
        }
        Err(e) => {
            error!("Error al escanear Wi-Fi: {:?}", e);
            vec![]
        }
    }
}

pub fn connect_wifi(wifi: &mut EspWifi, ssid: &str, pass: &str) -> anyhow::Result<()> {
    let _ = wifi.disconnect();
    let auth_method = if pass.is_empty() {
        info!("Red sin contraseña. Usando seguridad abierta...");
        AuthMethod::None
    } else {
        info!("Red con contraseña. Usando WPA/WPA2...");
        AuthMethod::WPAWPA2Personal
    };

    let wifi_config = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into()?,
        password: pass.try_into()?,
        auth_method,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;
    let _ = wifi.start(); 
    
    info!("🔄 Iniciando proceso de conexión a {}...", ssid);
    let _ = wifi.connect(); 
    
    for segundo in 1..=20 {
        sleep(Duration::from_secs(1));
        
        if wifi.is_connected().unwrap_or(false) {
            info!("✅ ¡Wi-Fi conectado a {} en {}s!", ssid, segundo);
            sleep(Duration::from_secs(2));
            return Ok(());
        }
    }
    
    anyhow::bail!("❌ No se pudo conectar a {} tras 20 segundos.", ssid);
}