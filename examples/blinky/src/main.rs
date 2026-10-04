//! Toggle the user LED (LD2 on PA5) every 200 ms.
//!
//! This program is the Rust counterpart of the STM32CubeL0 `GPIO_IOToggle`
//! example.
//!
//! | C example                                | This port                            |
//! |------------------------------------------|--------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`     | `board::init(SystemClock::Msi2MHz)`  |
//! | `HAL_GPIO_Init()` on PA5                 | `Led2::new(p.PA5)`                   |
//! | `HAL_Delay(100)`                         | `Timer::after_millis(100).await`     |
//! | `HAL_GPIO_TogglePin(GPIOA, GPIO_PIN_5)`  | `led.toggle()`                       |

#![no_std]
#![no_main]

use board::{Led2, SystemClock};
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = board::init(SystemClock::Msi2MHz);
    info!("blinky started");

    let mut led = Led2::new(p.PA5);

    // The C loop has a 100 ms delay before and after each toggle, so LD2
    // toggles every 200 ms.
    loop {
        Timer::after_millis(100).await;
        led.toggle();
        Timer::after_millis(100).await;
    }
}
