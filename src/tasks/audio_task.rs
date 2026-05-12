use log::info;

use std::thread;
use std::time::Duration;

pub fn start_audio_task() {

    thread::Builder::new()
        .name("audio_task".into())
        .stack_size(8 * 1024)
        .spawn(|| {

            loop {

                info!("Procesando audio FFT");

                thread::sleep(
                    Duration::from_millis(5)
                );
            }

        })
        .unwrap();
}