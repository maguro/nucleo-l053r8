# queues

In this example, a producer task sends an incrementing number to a consumer task through a queue. The queue holds one item.

- Each time the producer puts a number in the queue, it toggles the user LED (LD2 on PA5) and waits 1 second.
- The consumer checks that each number is one more than the number before it. It logs `consumer received N` for each correct number.
- If an error occurs, LD2 stays on, and the program logs an error.

The system clock is MSI at about 2.1 MHz.

## Origin

The C application is the STM32CubeL0 `FreeRTOS_Queues` application for NUCLEO-L053R8 (`Projects/NUCLEO-L053R8/Applications/FreeRTOS/FreeRTOS_Queues/Src/main.c`).

| C application | Rust port |
|---------------|-----------|
| `HAL_Init()` and `SystemClock_Config()`: MSI range 5, voltage range 3 | `board::init(SystemClock::Msi2MHz)` |
| `BSP_LED_Init(LED2)` | `Led2::new(p.PA5)` in a `Mutex` |
| `osMessageCreate()` with space for 1 item | `QUEUE`, a `Channel` with space for 1 item |
| `osThreadCreate()` for the consumer and the producer | `spawner.spawn()` for `consumer` and `producer` |
| `osMessagePut()` with a timeout of 100 ms | `with_timeout(TIMEOUT, queue.send(...))` |
| `osMessageGet()` with a timeout of 100 ms | `with_timeout(TIMEOUT, queue.receive())` |
| `ProducerValue` and `ConsumerValue` globals | `producer_value` and `consumer_value`, local variables of the tasks |

## Differences from FreeRTOS

1. **No globals for the debugger.** In C, a debugger watches the `ProducerValue` and `ConsumerValue` globals, and the two values must stay equal. In this port, each value is local to its task, and the consumer logs each number that it receives.
2. **No priorities.** The C readme says that the consumer has a higher priority than the producer. The C code gives both threads the same priority, `osPriorityBelowNormal`. All tasks in this port run in one embassy executor, which has no priorities. The order of events is the same in both programs. The consumer runs when the producer starts its wait of 1 second.
3. **The shared LED.** Both tasks change LD2, so LD2 is in an async `Mutex`. The [thread-creation README](../thread-creation/README.md) tells why the `Mutex` is in a `StaticCell`, and why the example depends on `portable-atomic`.

## Running it

Run these commands from the repository root:

```bash
cargo run -p queues
cargo run -p queues --release
```

The dev build uses about 31 KB of flash. The release build uses about 11 KB.

The [blinky README](../blinky/README.md) tells how to stop probe-rs, reconnect with `probe-rs attach`, and debug in RustRover. These steps also apply to queues. In the commands and paths, use `queues` in place of `blinky`.
