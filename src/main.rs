mod config;
mod discord;
mod tasks;
mod wifi;

use crate::config::env::AppConfig;
use crate::wifi::wifi::WifiCommand;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    dotenvy::dotenv().ok();

    let config = AppConfig::load()?;
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    let button = PinDriver::input(peripherals.pins.gpio0, Pull::Up)?;

    let _ = led.set_low();

    for _ in 0..3 {
        let _ = led.set_high();
        std::thread::sleep(Duration::from_millis(150));
        let _ = led.set_low();
        std::thread::sleep(Duration::from_millis(150));
    }

    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    let bot_active = Arc::new(AtomicBool::new(false));

    if let Err(e) = wifi::personal::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass) {
        error!("❌ Error en la conexión Wi-Fi inicial: {:?}", e);
        warn!("⏸️ El bot arrancará pausado.");
    } else {
        info!("✅ Wi-Fi conectado exitosamente.");
        bot_active.store(true, Ordering::SeqCst);
    }

    info!(
        "Bot iniciado. RAM libre inicial: {} bytes",
        unsafe { esp_idf_sys::esp_get_free_heap_size() }
    );

    let (wifi_tx, wifi_rx) = mpsc::channel();

    tasks::discord_task::start_discord_task(
        config.discord_token,
        config.discord_app_id,
        wifi_tx,
        Arc::clone(&bot_active),
    );

    let audio_level = Arc::new(AtomicU16::new(0));

    tasks::audio_task::start_audio_task(
        peripherals.adc1,
        peripherals.pins.gpio34,
        Arc::clone(&audio_level),
    );

    tasks::display_task::start_display_task(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Arc::clone(&audio_level),
    );

    info!("Tareas de audio y OLED iniciadas.");

    let mut loop_counter = 0;

    loop {
        if button.is_low() {
            let _ = led.set_high();
            info!("🔘 Botón presionado. Reintentando Wi-Fi...");
            bot_active.store(false, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(500));

            if let Err(e) = wifi::personal::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass) {
                error!("❌ Falló la reconexión: {:?}", e);
            } else {
                info!("✅ Wi-Fi reconectado.");
                bot_active.store(true, Ordering::SeqCst);
            }

            let _ = led.set_low();
        }

        match wifi_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(WifiCommand::Scan) => {
                let _ = led.set_high();
                let redes = wifi::personal::scan_wifi(&mut wifi);

                info!("Resultados del escaneo:");
                for red in redes {
                    info!("- {}", red);
                }

                let _ = led.set_low();
            }
            Ok(WifiCommand::Connect(nuevo_ssid, nueva_pass)) => {
                let _ = led.set_high();
                bot_active.store(false, Ordering::SeqCst);

                info!("Cambiar a red '{}'", nuevo_ssid);

                if let Err(e) = wifi::personal::connect_wifi(&mut wifi, &nuevo_ssid, &nueva_pass) {
                    error!("❌ Fallo al cambiar de Wi-Fi: {:?}", e);
                } else {
                    info!("✅ Cambio de Wi-Fi exitoso.");
                    bot_active.store(true, Ordering::SeqCst);
                }

                let _ = led.set_low();
            }
            Err(_) => {}
        }

        loop_counter += 1;
        if loop_counter >= 20 {
            info!(
                "Heap libre: {} | Mínimo histórico: {} | Audio: {}",
                unsafe { esp_idf_sys::esp_get_free_heap_size() },
                unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() },
                audio_level.load(Ordering::Relaxed),
            );
            loop_counter = 0;
        }
    }
}