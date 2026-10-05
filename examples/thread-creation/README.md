# thread-creation

In this example, two tasks share the user LED (LD2 on PA5). The tasks repeat a cycle of about 15 seconds:

1. For 5 seconds, both tasks run. Task 1 flashes LD2 three times every 1.5 seconds. Task 2 flashes LD2 two times every second.
2. Task 1 suspends itself. For 5 more seconds, only task 2 runs.
3. Task 2 resumes task 1 and suspends itself. For 5 seconds, task 1 turns LD2 on for 1 second and off for 0.5 seconds. Then task 1 resumes task 2, and the cycle starts again.

Each task checks the time only before each pass of its loop. Thus each part lasts until the last pass ends, and a full cycle takes about 16.5 seconds. The C application has the same structure.

The system clock is MSI at about 2.1 MHz. The program logs a line at startup and at each change of part.

## Origin

The C application is the STM32CubeL0 `FreeRTOS_ThreadCreation` application for NUCLEO-L053R8 (`Projects/NUCLEO-L053R8/Applications/FreeRTOS/FreeRTOS_ThreadCreation/Src/main.c`).

| C application | Rust port |
|---------------|-----------|
| `HAL_Init()` and `SystemClock_Config()`: MSI range 5, voltage range 3 | `board::init(SystemClock::Msi2MHz)` |
| `BSP_LED_Init(LED2)` | `Led2::new(p.PA5)` in a `Mutex` |
| `osThreadCreate()` for `LED_Thread1` and `LED_Thread2` | `spawner.spawn()` for `led_task1` and `led_task2` |
| `osDelay()` and `HAL_Delay()` | `Timer::after_millis().await` |
| `osKernelSysTick()` | `Instant::now()` |
| `osThreadSuspend(NULL)` | `resume_self.wait().await` |
| `osThreadResume()` | `resume_other.signal(())` |

## Differences from FreeRTOS

embassy is not an RTOS, so this port does three things differently:

1. **Suspend and resume.** embassy cannot suspend a task from outside. Each task has a `Signal`. A task suspends itself when it waits on its own `Signal`. The other task resumes the waiting task with a signal on that `Signal`.
2. **No preemption.** FreeRTOS can stop a thread at any time and run another thread. An embassy task runs until it reaches an `await` that must wait. The C threads busy-wait in `HAL_Delay()`, and FreeRTOS runs the other thread during that wait. In this port, a busy wait would stop the other task. Thus each wait is `Timer::after_millis().await`.
3. **The shared LED.** Both tasks change LD2, so LD2 is in an async `Mutex`. A task holds the lock only while it turns LD2 on or off. `main` creates the `Led2` at run time, and the tasks need a reference that lasts for the whole program. A `StaticCell` holds the `Mutex` for this reason.

`static_cell` needs atomic compare-and-swap, and the Cortex-M0+ does not have this instruction. Thus the example also depends on `portable-atomic` with its `critical-section` feature.

## Running it

Run these commands from the repository root:

```bash
cargo run -p thread-creation
cargo run -p thread-creation --release
```

The dev build uses about 31 KB of flash. The release build uses about 12 KB.

The [blinky README](../blinky/README.md) tells how to stop probe-rs, reconnect with `probe-rs attach`, and debug in RustRover. These steps also apply to thread-creation. In the commands and paths, use `thread-creation` in place of `blinky`.
