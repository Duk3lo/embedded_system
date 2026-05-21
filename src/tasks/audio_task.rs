// audio_task.rs
use esp_idf_svc::hal::adc::oneshot::{
    config::{AdcChannelConfig},
    AdcChannelDriver, AdcDriver,
};
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::Gpio34;
use log::info;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_audio_task(adc1: ADC1<'static>, mic_pin: Gpio34<'static>, audio_level: Arc<AtomicU16>) {
    thread::Builder::new()
        .name("audio_task".into())
        .stack_size(8 * 1024)
        .spawn(move || {
            info!("Iniciando tarea de audio...");

            let adc = Rc::new(AdcDriver::new(adc1).unwrap());

            let chan_cfg = AdcChannelConfig::default();

            let mut mic_channel = AdcChannelDriver::new(Rc::clone(&adc), mic_pin, &chan_cfg).unwrap();

            loop {
                let mut min_val: u16 = 4095;
                let mut max_val: u16 = 0;

                for _ in 0..100 {
                    let val: u16 = adc.read(&mut mic_channel).unwrap_or(0);
                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }
                }

                let amplitude: u16 = max_val.saturating_sub(min_val);
                audio_level.store(amplitude, Ordering::Relaxed);

                thread::sleep(Duration::from_millis(10));
            }
        })
        .unwrap();
}