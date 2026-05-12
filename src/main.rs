mod config;
mod discord;
mod tasks;
mod wifi_manager;

use crate::config::env::AppConfig;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use log::info;

fn main() -> anyhow::Result<()> {
    // 1. Inicialización básica
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    dotenvy::dotenv().ok();

    let config = AppConfig::load()?;
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // 2. Conectar Wi-Fi
    let mut wifi = EspWifi::new(peripherals.modem, sys_loop, Some(nvs))?;
    wifi_manager::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass)?;

    // 3. Lanzar Tareas (Multihilo)
    info!("Lanzando tareas del sistema...");
    
    // Tarea de Discord (20KB de Stack por HTTPS)
    tasks::discord_task::start_discord_task(
        config.discord_token.clone(),
        config.channel_id.clone()
    );

    //(8KB)
    //tasks::audio_task::start_audio_task();

    //(6KB)
    //tasks::display_task::start_display_task();

    // 4. Bucle principal (Monitor de sistema)
    loop {
        info!("Sistema OK - RAM Libre: {} bytes", unsafe { esp_idf_sys::esp_get_free_heap_size() });
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}