//! Toggle the user LED (LD2 on PA5) once per second from the TIM6 interrupt.
//!
//! This program is the Rust counterpart of the STM32CubeL0 `TIM_TimeBase`
//! example.
//!
//! | C example                                         | This port                                          |
//! |---------------------------------------------------|----------------------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`              | `board::init(SystemClock::Pll32MHz)`               |
//! | `BSP_LED_Init(LED2)`                              | `Led2::new(p.PA5)`                                 |
//! | `HAL_TIM_Base_Init()`                             | `Timer::new(p.TIM6)`, `set_tick_freq()`, `ARR`     |
//! | `HAL_TIM_Base_Start_IT()`                         | `enable_update_interrupt()`, `enable()`, `start()` |
//! | `TIM6_DAC_IRQHandler()`, `HAL_TIM_IRQHandler()`   | `TIM6_DAC()`                                       |
//! | `HAL_TIM_PeriodElapsedCallback()`                 | the body of the `main()` loop                      |
//!
//! embassy-stm32 has no async function that waits for the update event of a
//! basic timer. Because of this, this program has its own interrupt handler,
//! `TIM6_DAC`. The handler clears the update flag of TIM6 and signals `main`.
//! Then `main` toggles LD2.

#![no_std]
#![no_main]

use board::{Led2, SystemClock};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::interrupt::InterruptExt;
use embassy_stm32::pac::{self, interrupt};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::Timer;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use {defmt_rtt as _, panic_probe as _};

/// `COUNTER_HZ` is the frequency of the TIM6 counter. The C example uses the
/// same value.
const COUNTER_HZ: u32 = 10_000;

/// `PERIOD` is the number of counts in one update period of TIM6. At
/// `COUNTER_HZ`, TIM6 makes one update event each second.
const PERIOD: u16 = 10_000;

/// `PERIOD_ELAPSED` carries a signal from `TIM6_DAC` to `main` at each update
/// event of TIM6.
static PERIOD_ELAPSED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = board::init(SystemClock::Pll32MHz);
    info!("tim-timebase started");

    let mut led = Led2::new(p.PA5);

    // The prescaler divides the 32 MHz timer clock to 10 kHz (PSC = 3199).
    // TIM6 then makes an update event after each 10000 counts (ARR = 9999).
    let mut timer = Timer::new(p.TIM6);
    timer.set_tick_freq(Hertz(COUNTER_HZ));
    timer.regs_basic().arr().write(|w| w.set_arr(PERIOD - 1));

    // `set_tick_freq` makes an update event to load the prescaler. Clear its
    // flag, so that the first interrupt comes after one full period.
    timer.clear_update_interrupt();
    timer.enable_update_interrupt(true);

    // SAFETY: After this call, the TIM6_DAC handler can run at any time. No
    // code in this program expects this interrupt to stay disabled, and the
    // handler only clears a timer flag and signals `PERIOD_ELAPSED`.
    unsafe { interrupt::TIM6_DAC.enable() };
    timer.start();

    loop {
        PERIOD_ELAPSED.wait().await;
        led.toggle();
    }
}

/// `TIM6_DAC` handles the TIM6 interrupt. It clears the update flag, as
/// `HAL_TIM_IRQHandler()` does in C. Then it signals `main`.
#[interrupt]
fn TIM6_DAC() {
    pac::TIM6.sr().modify(|w| w.set_uif(false));
    PERIOD_ELAPSED.signal(());
}
