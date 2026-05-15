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
use std::sync::atomic::{AtomicBool, Ordering};
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

    // 💡 INICIALIZAR EL LED (GPIO 2)
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    let _ = led.set_low();

    // 🔘 INICIALIZAR EL BOTÓN (GPIO 0 - Usualmente el botón BOOT del ESP32)
    let button = PinDriver::input(peripherals.pins.gpio0, Pull::Up)?;

    // 💡 PARPADEO DE INICIO
    for _ in 0..3 {
        let _ = led.set_high();
        std::thread::sleep(Duration::from_millis(150));
        let _ = led.set_low();
        std::thread::sleep(Duration::from_millis(150));
    }

    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    
    // 🚦 BANDERA DE ESTADO PARA TAREAS DE INTERNET
    let bot_active = Arc::new(AtomicBool::new(false));

    // INTENTO DE CONEXIÓN INICIAL
    if let Err(e) = wifi::personal::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass) {
        error!("❌ Error en la conexión Wi-Fi inicial: {:?}", e);
        warn!("⏸️ El bot iniciará pausado. Presiona el botón para reintentar.");
        // bot_active ya es false
    } else {
        info!("✅ Wi-Fi conectado exitosamente en el arranque.");
        bot_active.store(true, Ordering::SeqCst);
    }

    info!("Bot iniciado. RAM libre inicial: {} bytes", unsafe {
        esp_idf_sys::esp_get_free_heap_size()
    });

    // Canal de comunicación para recibir órdenes desde Discord
    let (wifi_tx, wifi_rx) = mpsc::channel();

    // Iniciamos la tarea de Discord pasándole la bandera de estado
    tasks::discord_task::start_discord_task(
        config.discord_token,
        config.discord_app_id,
        wifi_tx,
        Arc::clone(&bot_active),
    );

    let mut loop_counter = 0;

    loop {
        if button.is_low() {
            let _ = led.set_high();
            info!("🔘 Botón presionado! Intentando conectar/reconectar Wi-Fi...");
            bot_active.store(false, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(500)); // Debounce del botón

            if let Err(e) = wifi::personal::connect_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_pass) {
                error!("❌ Falló la reconexión: {:?}", e);
            } else {
                info!("✅ Wi-Fi reconectado. Reactivando tareas de internet...");
                bot_active.store(true, Ordering::SeqCst); // Despierta el hilo de Discord
            }
            let _ = led.set_low();
        }

        // Escuchamos si Discord manda alguna orden (timeout rápido para poder leer el botón)
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
                bot_active.store(false, Ordering::SeqCst); // Pausa el bot al cambiar de red
                info!("Petición desde Discord: Cambiando red a '{}'...", nuevo_ssid);
                
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

        // Mostrar RAM libre cada ~10 segundos (20 iteraciones de 500ms)
        loop_counter += 1;
        if loop_counter >= 20 {
            info!("Heap libre: {} | Mínimo histórico: {}", 
                unsafe { esp_idf_sys::esp_get_free_heap_size() },
                unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() }
            );
            loop_counter = 0;
        }
    }
}