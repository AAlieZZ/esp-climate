mod kanji;

use std::{sync::Arc, time::Duration};

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};

use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use kanji::Kanji;
// get dot-matrix by https://www.eecso.com/test/zimo/
const 田: &[u8] = &[
    0x00,0x00,0xfe,0x82,0x82,0x82,0x82,0xfe,0x82,0x82,0x82,0x82,0xfe,0x00,0x00,0x00,0x00,0x00,0x7f,0x20,0x20,0x20,0x20,0x3f,0x20,0x20,0x20,0x20,0x7f,0x00,0x00,0x00
];
const 所: &[u8] = &[
    0x00,0x00,0xfc,0x24,0x24,0x22,0xe3,0x02,0xfc,0x44,0x44,0x42,0xc3,0x42,0x40,0x00,0x40,0x30,0x0f,0x02,0x02,0x82,0x43,0x30,0x0f,0x00,0x00,0x00,0xff,0x00,0x00,0x00
];
const 浩: &[u8] = &[
    0x10,0x60,0x02,0x8c,0x20,0x90,0x8e,0x88,0x88,0xff,0x88,0x88,0x88,0x88,0x80,0x00,0x04,0x04,0x7e,0x01,0x00,0x00,0xfc,0x44,0x44,0x44,0x44,0x44,0xfc,0x00,0x00,0x00
];

fn cycle_duty(mut pwm: LedcDriver, sleep: Duration) -> anyhow::Result<()> {
    let max_duty = pwm.get_max_duty();

    loop {
        for numerator in 0..=max_duty {
            pwm.set_duty(numerator)?;
            std::thread::sleep(sleep);
        }
        for numerator in 0..=max_duty {
            pwm.set_duty(max_duty - numerator)?;
            std::thread::sleep(sleep);
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    println!("Setting up PWM output channels");

    let peripherals = Peripherals::take()?;
    let led_config = config::TimerConfig::new().frequency(25.kHz().into());
    let timer = Arc::new(LedcTimerDriver::new(peripherals.ledc.timer0, &led_config)?);
    let channel0 = LedcDriver::new(peripherals.ledc.channel0, timer.clone(), peripherals.pins.gpio25)?;
    let channel1 = LedcDriver::new(peripherals.ledc.channel1, timer.clone(), peripherals.pins.gpio26)?;
    let channel2 = LedcDriver::new(peripherals.ledc.channel2, timer, peripherals.pins.gpio27)?;

    println!("Spawning PWM threads");

    // let thread0 = 
    std::thread::Builder::new().stack_size(7000).spawn(move || cycle_duty(channel0, Duration::from_millis(10)))?;
    std::thread::Builder::new().stack_size(7000).spawn(move || cycle_duty(channel1, Duration::from_millis(20)))?;
    std::thread::Builder::new().stack_size(7000).spawn(move || cycle_duty(channel2, Duration::from_millis(40)))?;

    let mut dht11 = dht11::Dht11::new(PinDriver::input_output_od(peripherals.pins.gpio13)?);
    let mut dht11_delay = esp_idf_hal::delay::Ets;

    let i2c = I2cDriver::new(peripherals.i2c0, peripherals.pins.gpio21, peripherals.pins.gpio22, &I2cConfig::new().baudrate(100.kHz().into()))?;
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().expect("display error");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();

    loop {
        match dht11.perform_measurement(&mut dht11_delay) {
            Ok(meas) => {
                while Text::with_baseline(&format!("Temp: {}.{}`C", meas.temperature/10, meas.temperature%10), Point::zero(), text_style, Baseline::Top).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                while Text::with_baseline(&format!("Hum: {}.{}%", meas.humidity/10, meas.humidity%10), Point::new(0, 20), text_style, Baseline::Top).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                while Text::with_baseline("114__514", Point::new(0, 49), text_style, Baseline::Top).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                while Kanji::dot_matrix(80, 48, 田).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                while Kanji::dot_matrix(96, 48, 所).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                while Kanji::dot_matrix(112, 48, 浩).draw(&mut display).is_err() {
                    eprintln!("display error")
                }
                display.flush().unwrap();
            },
            Err(e) => eprintln!("Error: {:?}", e),
        };
        std::thread::sleep(Duration::from_millis(2048));
        display.clear_buffer();
    }

    // println!("Waiting for PWM threads");

    // thread0.join().unwrap()?;

    // println!("Joined PWM threads");

    // println!("Done");
}
