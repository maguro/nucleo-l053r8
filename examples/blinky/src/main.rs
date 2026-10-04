//! Blink the user LED (LD2 on PA5) once per second.
//!
//! This program is the Rust counterpart of the STM32CubeL0 `GPIO_IOToggle`
//! example.

#![no_std]
#![no_main]

use board::{Led2, SystemClock};
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = board::init(SystemClock::Msi);
    info!("blinky started");

    let mut led = Led2::new(p.PA5);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
