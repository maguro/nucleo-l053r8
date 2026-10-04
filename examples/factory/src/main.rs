//! Toggle the user LED (LD2 on PA5) at one of three speeds. Each press of the
//! user button (B1 on PC13) selects the next speed.
//!
//! This program is the Rust counterpart of the factory demo that ST installs
//! on the board. That demo is the STM32CubeL0 `Demonstrations` project for
//! NUCLEO-L053R8. This port covers only the no-shield path of the demo. The C
//! demo also drives an Adafruit 1.8" TFT shield when it detects one on PB0.
//!
//! | C demo                                      | This port                                |
//! |---------------------------------------------|------------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`        | `board::init(SystemClock::Pll32MHz)`     |
//! | `BSP_LED_Init(LED2)`                        | `Led2::new(p.PA5)`                       |
//! | `BSP_PB_Init(BUTTON_KEY, BUTTON_MODE_EXTI)` | `UserButton::new(p.PC13, p.EXTI13)`      |
//! | `LED2_Blink()` loop                         | `main()` loop                            |
//! | `HAL_GPIO_EXTI_Callback()`                  | `button_task()`, which sends a `ButtonPress` |
//! | `BlinkSpeed` global                         | `speed`, a local variable in `main()`    |
//!
//! In the C demo, the interrupt callback and the main loop share the
//! `BlinkSpeed` global. In this port, only `main` owns the blink speed.
//! `button_task` sends one `ButtonPress` message to `main` for each press of
//! B1. Because of this, no other code can change the blink speed.

#![no_std]
#![no_main]

use board::{Led2, SystemClock, UserButton};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// `TOGGLE_DELAY_MS` holds the delay between LED toggles for each blink speed,
/// in milliseconds. The values are the same as in the C demo.
const TOGGLE_DELAY_MS: [u64; 3] = [500, 100, 50];

/// `PRESS_QUEUE_LEN` is the number of presses that `PRESSES` can hold before
/// `main` takes them.
const PRESS_QUEUE_LEN: usize = 4;

/// `ButtonPress` is the message that `button_task` sends for one press of B1.
struct ButtonPress;

/// `PRESSES` carries `ButtonPress` messages from `button_task` to `main`.
/// `CriticalSectionRawMutex` makes the channel safe to use from any task or
/// interrupt.
static PRESSES: Channel<CriticalSectionRawMutex, ButtonPress, PRESS_QUEUE_LEN> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = board::init(SystemClock::Pll32MHz);
    info!("factory demo started");

    let button = UserButton::new(p.PC13, p.EXTI13);
    spawner.spawn(button_task(button, PRESSES.sender()).unwrap());
    let presses = PRESSES.receiver();

    let mut led = Led2::new(p.PA5);

    // `speed` is an index into `TOGGLE_DELAY_MS`. Only this loop changes
    // `speed`.
    let mut speed = 0;

    loop {
        led.toggle();
        Timer::after_millis(TOGGLE_DELAY_MS[speed]).await;

        // Select the next speed for each press that arrived during the delay.
        // After the last speed, the selection goes back to the first speed.
        while presses.try_receive().is_ok() {
            speed = (speed + 1) % TOGGLE_DELAY_MS.len();
            info!(
                "blink speed {}: toggle every {} ms",
                speed, TOGGLE_DELAY_MS[speed]
            );
        }
    }
}

/// `button_task` sends one `ButtonPress` to `main` for each press of B1.
#[embassy_executor::task]
async fn button_task(
    mut button: UserButton<'static>,
    presses: Sender<'static, CriticalSectionRawMutex, ButtonPress, PRESS_QUEUE_LEN>,
) {
    loop {
        button.wait_for_press().await;

        // If the queue is full, `send` waits until `main` takes a press.
        // `button_task` does not detect presses of B1 during that wait.
        presses.send(ButtonPress).await;
    }
}
