//! Toggle the user LED (LD2 on PA5) on each press of the user button (B1 on
//! PC13).
//!
//! This program is the Rust counterpart of the STM32CubeL0 `GPIO_EXTI`
//! example.
//!
//! | C example                             | This port                            |
//! |---------------------------------------|--------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`  | `board::init(SystemClock::Msi2MHz)`  |
//! | `BSP_LED_Init(LED2)`                  | `Led2::new(p.PA5)`                   |
//! | `EXTILine4_15_Config()`               | `UserButton::new(p.PC13, p.EXTI13)`  |
//! | `HAL_GPIO_EXTI_Callback()`            | the body of the `main()` loop        |
//!
//! In the C example, the interrupt callback toggles LD2. In this port, the
//! interrupt wakes `main`, and `main` toggles LD2.

#![no_std]
#![no_main]

use board::{Led2, SystemClock, UserButton};
use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = board::init(SystemClock::Msi2MHz);
    info!("gpio-exti started");

    let mut led = Led2::new(p.PA5);
    let mut button = UserButton::new(p.PC13, p.EXTI13);

    loop {
        button.wait_for_press().await;
        led.toggle();
    }
}
