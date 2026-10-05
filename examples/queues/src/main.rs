//! A producer task sends an incrementing number to a consumer task through a
//! queue that holds one item.
//!
//! This program is the Rust counterpart of the STM32CubeL0 `FreeRTOS_Queues`
//! application. Each time the producer puts a number in the queue, it toggles
//! the user LED (LD2 on PA5) and waits 1 second. The consumer checks that it
//! receives each number in sequence. If an error occurs, LD2 stays on.
//!
//! | C application                         | This port                                   |
//! |---------------------------------------|---------------------------------------------|
//! | `HAL_Init()`, `SystemClock_Config()`  | `board::init(SystemClock::Msi2MHz)`         |
//! | `BSP_LED_Init(LED2)`                  | `Led2::new(p.PA5)` in a `Mutex`             |
//! | `osMessageCreate()`                   | `QUEUE`, a `Channel` with space for 1 item  |
//! | `osThreadCreate()`                    | `spawner.spawn()`                           |
//! | `osMessagePut()` with a timeout       | `with_timeout(TIMEOUT, sender.send())`      |
//! | `osMessageGet()` with a timeout       | `with_timeout(TIMEOUT, receiver.receive())` |
//! | `ProducerValue`, `ConsumerValue`      | local variables of the two tasks            |
//!
//! In C, a debugger watches the `ProducerValue` and `ConsumerValue` globals.
//! In this port, each value is local to its task, and the consumer logs each
//! number that it receives.

#![no_std]
#![no_main]

use board::{Led2, SystemClock};
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer, with_timeout};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

/// `QUEUE_SIZE` is the number of items that `QUEUE` can hold. The C
/// application uses the same value.
const QUEUE_SIZE: usize = 1;

/// `TIMEOUT` is the time that each task waits for the queue. The C
/// application uses the same value.
const TIMEOUT: Duration = Duration::from_millis(100);

/// `SharedLed` lets both tasks use LD2. A task holds the lock only while it
/// changes LD2.
type SharedLed = Mutex<CriticalSectionRawMutex, Led2<'static>>;

/// `LED` holds the `SharedLed`. `main` creates the `Led2` at run time, so
/// `LED` is a `StaticCell` that `main` fills one time.
static LED: StaticCell<SharedLed> = StaticCell::new();

/// `QUEUE` carries the numbers from `producer` to `consumer`.
static QUEUE: Channel<CriticalSectionRawMutex, u32, QUEUE_SIZE> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = board::init(SystemClock::Msi2MHz);
    info!("queues started");

    let led: &'static SharedLed = LED.init(Mutex::new(Led2::new(p.PA5)));

    spawner.spawn(consumer(led, QUEUE.receiver()).unwrap());
    spawner.spawn(producer(led, QUEUE.sender()).unwrap());
}

/// `producer` is the counterpart of `MessageQueueProducer()` in C.
#[embassy_executor::task]
async fn producer(
    led: &'static SharedLed,
    queue: Sender<'static, CriticalSectionRawMutex, u32, QUEUE_SIZE>,
) {
    let mut producer_value: u32 = 0;
    loop {
        match with_timeout(TIMEOUT, queue.send(producer_value)).await {
            Ok(()) => {
                producer_value += 1;
                led.lock().await.toggle();
                Timer::after_millis(1000).await;
            }
            Err(_) => {
                error!(
                    "producer: the queue stayed full for {} ms",
                    TIMEOUT.as_millis()
                );
                led.lock().await.on();
            }
        }
    }
}

/// `consumer` is the counterpart of `MessageQueueConsumer()` in C.
#[embassy_executor::task]
async fn consumer(
    led: &'static SharedLed,
    queue: Receiver<'static, CriticalSectionRawMutex, u32, QUEUE_SIZE>,
) {
    let mut consumer_value: u32 = 0;
    loop {
        // A timeout is not an error. The consumer then waits again.
        if let Ok(value) = with_timeout(TIMEOUT, queue.receive()).await {
            if value == consumer_value {
                info!("consumer received {}", value);
                consumer_value += 1;
            } else {
                error!("consumer: expected {}, received {}", consumer_value, value);
                consumer_value = value;
                led.lock().await.on();
            }
        }
    }
}
