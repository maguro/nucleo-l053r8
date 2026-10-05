//! Two tasks share the user LED (LD2 on PA5) in a cycle of 15 seconds.
//!
//! This program is the Rust counterpart of the STM32CubeL0
//! `FreeRTOS_ThreadCreation` application. The cycle has three parts:
//!
//! 1. For 5 seconds, both tasks run. Task 1 flashes LD2 three times every
//!    1.5 seconds. Task 2 flashes LD2 two times every second.
//! 2. Task 1 suspends itself. For 5 more seconds, only task 2 runs.
//! 3. Task 2 resumes task 1 and suspends itself. For 5 seconds, task 1 turns
//!    LD2 on for 1 second and off for 0.5 seconds. Then task 1 resumes task 2,
//!    and the cycle starts again.
//!
//! | C application                         | This port                                   |
//! |---------------------------------------|---------------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`  | `board::init(SystemClock::Msi2MHz)`         |
//! | `BSP_LED_Init(LED2)`                  | `Led2::new(p.PA5)` in a `Mutex`             |
//! | `osThreadCreate()`                    | `spawner.spawn()`                           |
//! | `osDelay()`, `HAL_Delay()`            | `Timer::after_millis().await`               |
//! | `osKernelSysTick()`                   | `Instant::now()`                            |
//! | `osThreadSuspend(NULL)`               | `wait()` on the `Signal` of the task        |
//! | `osThreadResume()`                    | `signal()` on the `Signal` of the other task |
//!
//! embassy cannot suspend a task from outside. Instead, a task that suspends
//! itself waits on its own `Signal`. The other task resumes the waiting task
//! with a signal on that `Signal`. embassy also does not preempt a task. In C, `HAL_Delay()`
//! busy-waits, and FreeRTOS lets the other thread run during that wait. In
//! this port, each wait is `Timer::after_millis().await`, so that the other
//! task can run.

#![no_std]
#![no_main]

use board::{Led2, SystemClock};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

/// `SharedLed` lets both tasks use LD2. A task holds the lock only while it
/// turns LD2 on or off.
type SharedLed = Mutex<CriticalSectionRawMutex, Led2<'static>>;

/// `Resume` is the signal that resumes a task that suspended itself.
type Resume = Signal<CriticalSectionRawMutex, ()>;

/// `LED` holds the `SharedLed`. `main` creates the `Led2` at run time, so
/// `LED` is a `StaticCell` that `main` fills one time.
static LED: StaticCell<SharedLed> = StaticCell::new();

/// `RESUME_TASK1` resumes `led_task1`.
static RESUME_TASK1: Resume = Signal::new();

/// `RESUME_TASK2` resumes `led_task2`.
static RESUME_TASK2: Resume = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = board::init(SystemClock::Msi2MHz);
    info!("thread-creation started");

    let led: &'static SharedLed = LED.init(Mutex::new(Led2::new(p.PA5)));

    spawner.spawn(led_task1(led, &RESUME_TASK1, &RESUME_TASK2).unwrap());
    spawner.spawn(led_task2(led, &RESUME_TASK2, &RESUME_TASK1).unwrap());
}

/// `led_task1` is the counterpart of `LED_Thread1()` in C.
#[embassy_executor::task]
async fn led_task1(
    led: &'static SharedLed,
    resume_self: &'static Resume,
    resume_other: &'static Resume,
) {
    loop {
        // For 5 seconds, flash LD2 three times, then wait 1.5 seconds.
        let end = Instant::now() + Duration::from_millis(5000);
        while Instant::now() <= end {
            for _ in 0..2 {
                led.lock().await.on();
                Timer::after_millis(80).await;
                led.lock().await.off();
                Timer::after_millis(80).await;
            }
            led.lock().await.on();
            Timer::after_millis(80).await;
            led.lock().await.off();
            Timer::after_millis(1500).await;
        }
        led.lock().await.off();

        info!("task 1 suspends itself");
        resume_self.wait().await;

        // For 5 seconds, turn LD2 on for 1 second and off for 0.5 seconds.
        let end = Instant::now() + Duration::from_millis(5000);
        while Instant::now() <= end {
            led.lock().await.on();
            Timer::after_millis(1000).await;
            led.lock().await.off();
            Timer::after_millis(500).await;
        }

        info!("task 1 resumes task 2");
        resume_other.signal(());
    }
}

/// `led_task2` is the counterpart of `LED_Thread2()` in C.
#[embassy_executor::task]
async fn led_task2(
    led: &'static SharedLed,
    resume_self: &'static Resume,
    resume_other: &'static Resume,
) {
    loop {
        // For 10 seconds, flash LD2 two times, then wait 1 second.
        let end = Instant::now() + Duration::from_millis(10_000);
        while Instant::now() <= end {
            led.lock().await.on();
            Timer::after_millis(100).await;
            led.lock().await.off();
            Timer::after_millis(100).await;
            led.lock().await.on();
            Timer::after_millis(100).await;
            led.lock().await.off();
            Timer::after_millis(1000).await;
        }
        led.lock().await.off();

        info!("task 2 resumes task 1 and suspends itself");
        resume_other.signal(());
        resume_self.wait().await;
    }
}
