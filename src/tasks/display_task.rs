use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::gpio::{
    AnyIOPin, AnyOutputPin, Gpio16, Gpio17, Gpio18, Gpio23, PinDriver,
};
use esp_idf_svc::hal::spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriverConfig, SPI2};
use esp_idf_svc::hal::units::FromValueType;
use log::info;
use ssd1306::{prelude::*, Ssd1306};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_display_task(
    spi: SPI2<'static>,
    sclk: Gpio18<'static>,
    sdo: Gpio23<'static>,
    dc_pin: Gpio17<'static>,
    rst_pin: Gpio16<'static>,
    audio_level: Arc<AtomicU16>,
) {
    thread::Builder::new()
        .name("display_task".into())
        .stack_size(10 * 1024)
        .spawn(move || {
            info!("Iniciando configuración del Display OLED...");

            let spi_config = SpiConfig::new().baudrate(10.MHz().into());
            let driver_config = SpiDriverConfig::new();

            let spi_device = SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Option::<AnyIOPin>::None,
                Option::<AnyOutputPin>::None,
                &driver_config,
                &spi_config,
            )
            .unwrap();

            let dc = PinDriver::output(dc_pin).unwrap();
            let mut rst = PinDriver::output(rst_pin).unwrap();

            let interface = SPIInterface::new(spi_device, dc);
            let mut display =
                Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
                    .into_buffered_graphics_mode();

            rst.set_high().unwrap();
            thread::sleep(Duration::from_millis(10));
            rst.set_low().unwrap();
            thread::sleep(Duration::from_millis(10));
            rst.set_high().unwrap();

            display.init().unwrap();

            let bar_style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();

            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

            loop {
                let level = audio_level.load(Ordering::Relaxed);

                // Si quieres una barra más visible, la escalamos a 0..64
                let mapped_height = ((level as u32 * 64) / 4095).clamp(1, 64) as i32;

                display.clear(BinaryColor::Off).unwrap();

                // Texto arriba
                let text = format!("MIC: {}", level);
                Text::new(&text, Point::new(0, 12), text_style)
                    .draw(&mut display)
                    .unwrap();

                // Barra abajo
                Rectangle::new(
                    Point::new(54, 64 - mapped_height),
                    Size::new(20, mapped_height as u32),
                )
                .into_styled(bar_style)
                .draw(&mut display)
                .unwrap();

                display.flush().unwrap();
                thread::sleep(Duration::from_millis(33));
            }
        })
        .unwrap();
}