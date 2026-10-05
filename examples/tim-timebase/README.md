# tim-timebase

This example uses the TIM6 timer to toggle the user LED (LD2 on PA5) once per second. The system clock runs at 32 MHz. The program logs `tim-timebase started` at startup.

## Origin

The C example is the STM32CubeL0 `TIM_TimeBase` example for NUCLEO-L053R8 (`Projects/NUCLEO-L053R8/Examples/TIM/TIM_TimeBase/Src/main.c`).

| C example | Rust port |
|-----------|-----------|
| `HAL_Init()` and `SystemClock_Config()`: HSI16 through the PLL, x4 / 2 | `board::init(SystemClock::Pll32MHz)` |
| `BSP_LED_Init(LED2)` | `Led2::new(p.PA5)` |
| `HAL_TIM_Base_Init()`: prescaler 3199, period 9999 | `Timer::new(p.TIM6)`, `set_tick_freq(Hertz(10_000))`, and a write to `ARR` |
| `HAL_TIM_Base_Start_IT()` | `enable_update_interrupt(true)`, `TIM6_DAC.enable()`, and `start()` |
| `TIM6_DAC_IRQHandler()` and `HAL_TIM_IRQHandler()` | `TIM6_DAC()`, which clears the update flag |
| `HAL_TIM_PeriodElapsedCallback()`, which toggles LED2 | the body of the `main()` loop |

TIM6 counts at 10 kHz. After 10000 counts, TIM6 makes an update event and starts again. Thus TIM6 makes one update event each second. embassy-time uses a different timer, TIM22, so the two timers do not interfere.

## The interrupt handler

embassy-stm32 has no async function that waits for the update event of a basic timer such as TIM6. Because of this, this example has its own interrupt handler, `TIM6_DAC`. The `#[interrupt]` attribute from cortex-m-rt connects the function to the interrupt.

The handler does two things:

1. It clears the update flag of TIM6. If the flag stays set, the interrupt occurs again immediately.
2. It signals `PERIOD_ELAPSED`. `main` waits on this `Signal`, and it toggles LD2 when the signal arrives.

The handler writes the TIM6 status register directly. embassy-stm32 gives access to the registers only through its `unstable-pac` feature, and the root `Cargo.toml` turns on this feature. embassy can change this register API in a future version.

`main` enables the interrupt in an `unsafe` block. After this call, the handler can run at any time. The `SAFETY` comment in `main.rs` tells why this use is safe. In the other examples, embassy drivers enable their own interrupts, so the examples do not need `unsafe`.

## Running it

Run these commands from the repository root:

```bash
cargo run -p tim-timebase
cargo run -p tim-timebase --release
```

The dev build uses about 24 KB of flash. The release build uses about 9 KB.

The [blinky README](../blinky/README.md) tells how to stop probe-rs, reconnect with `probe-rs attach`, and debug in RustRover. These steps also apply to tim-timebase. In the commands and paths, use `tim-timebase` in place of `blinky`.
