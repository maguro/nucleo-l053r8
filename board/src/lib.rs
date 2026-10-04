//! This crate gives board support for the NUCLEO-L053R8.
//!
//! This crate does the same job as ST's board support package (BSP). The BSP
//! is the `stm32l0xx_nucleo.c` driver that the STM32CubeL0 examples call. This
//! crate sets up the MCU pins that the board connects to its LED and its
//! button. Because of this, the examples do not set up these pins and their
//! interrupts.
//!
//! | ST BSP (C)                               | This crate                         |
//! |------------------------------------------|------------------------------------|
//! | `HAL_Init()` and `SystemClock_Config()`  | [`init`]                           |
//! | `BSP_LED_Init(LED2)`                     | [`Led2::new`]                      |
//! | `BSP_LED_On`, `BSP_LED_Off`, `BSP_LED_Toggle` | [`Led2::on`], [`Led2::off`], [`Led2::toggle`] |
//! | `BSP_PB_Init(BUTTON_KEY, BUTTON_MODE_EXTI)` | [`UserButton::new`]             |
//! | `BSP_PB_GetState(BUTTON_KEY)`            | [`UserButton::is_pressed`]         |
//!
//! In C, the button interrupt calls `HAL_GPIO_EXTI_Callback()`, and the
//! program reacts to the press in that function. With this crate, an async
//! task awaits [`UserButton::wait_for_press`] instead of a callback.
//!
//! Each constructor takes the exact pin types that it needs, such as
//! `Peri<'d, PA5>` for LD2. If you give a constructor a different pin, the
//! compiler reports an error. A program can give each pin to only one driver.

#![no_std]

use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{EXTI13, PA5, PC13};
use embassy_stm32::rcc::{Pll, PllDiv, PllMul, PllSource, Sysclk};
use embassy_stm32::{Config, Peri, Peripherals, bind_interrupts, interrupt};

// EXTI lines 4 to 15 share one interrupt on the L0. B1 is on line 13. Because
// this crate binds EXTI4_15, an example that uses this crate must not bind
// EXTI4_15 again. Other pins on EXTI lines 4 to 15 also use this binding.
bind_interrupts!(struct Irqs {
    EXTI4_15 => exti::InterruptHandler<interrupt::typelevel::EXTI4_15>;
});

/// `SystemClock` selects the system clock that [`init`] sets up.
#[derive(Clone, Copy)]
pub enum SystemClock {
    /// `Msi` runs the core from MSI at about 4.2 MHz. embassy-stm32 uses this
    /// clock by default. After a reset, the chip starts from MSI at about
    /// 2.1 MHz.
    Msi,
    /// `Pll32MHz` runs the core at 32 MHz from HSI16 through the PLL
    /// (16 MHz x 4 / 2). 32 MHz is the maximum frequency of the chip. The C
    /// factory demo sets the same clock in `SystemClock_Config()`.
    Pll32MHz,
}

/// `init` starts the chip with the given system clock. Then `init` returns the
/// peripherals of the chip.
///
/// embassy-stm32 also sets the core voltage range and the flash wait states
/// that the clock frequency needs.
pub fn init(clock: SystemClock) -> Peripherals {
    let mut config = Config::default();
    if let SystemClock::Pll32MHz = clock {
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            mul: PllMul::MUL4,
            div: PllDiv::DIV2,
        });
        config.rcc.sys = Sysclk::PLL1_R;
    }
    embassy_stm32::init(config)
}

/// `Led2` controls the green user LED, LD2, on PA5. LD2 is on while PA5 is
/// high.
pub struct Led2<'d> {
    pin: Output<'d>,
}

impl<'d> Led2<'d> {
    /// `new` configures PA5 as an output. LD2 is off when `new` returns.
    pub fn new(pin: Peri<'d, PA5>) -> Self {
        Self {
            pin: Output::new(pin, Level::Low, Speed::Low),
        }
    }

    /// `on` turns LD2 on.
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// `off` turns LD2 off.
    pub fn off(&mut self) {
        self.pin.set_low();
    }

    /// `toggle` turns LD2 on if LD2 is off. `toggle` turns LD2 off if LD2 is on.
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }
}

/// `UserButton` reads the blue user button, B1, on PC13.
///
/// The board has an external pull-up resistor on PC13. When you press B1, B1
/// connects PC13 to ground. As a result, PC13 is low while you hold B1 down.
pub struct UserButton<'d> {
    pin: ExtiInput<'d, Async>,
}

impl<'d> UserButton<'d> {
    /// `new` configures PC13 as an input with an interrupt on EXTI line 13.
    pub fn new(pin: Peri<'d, PC13>, exti: Peri<'d, EXTI13>) -> Self {
        Self {
            pin: ExtiInput::new(pin, exti, Pull::None, Irqs),
        }
    }

    /// `is_pressed` returns `true` while you hold B1 down.
    pub fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }

    /// `wait_for_press` waits until you press B1. A press changes PC13 from
    /// high to low. The core can sleep while `wait_for_press` waits.
    pub async fn wait_for_press(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }
}
