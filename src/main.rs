mod config;
mod discord;
mod tasks;
mod wifi;

use crate::config::env::AppConfig;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    dotenvy::dotenv().ok();

    let config = AppConfig::load()?;
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    wifi::personal::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass)?;

    info!("Lanzando tareas del sistema...");

    tasks::discord_task::start_discord_task(
        config.discord_token.clone(),
        config.channel_id.clone(),
    );

    loop {
        info!(
            "Sistema OK - RAM Libre: {} bytes",
            unsafe { esp_idf_sys::esp_get_free_heap_size() }
        );
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}