use log::info;

use std::thread;
use std::time::Duration;

pub fn start_display_task() {

    thread::Builder::new()
        .name("display_task".into())
        .stack_size(6 * 1024)
        .spawn(|| {

            loop {

                info!("Actualizando display");

                thread::sleep(
                    Duration::from_millis(16)
                );
            }

        })
        .unwrap();
}