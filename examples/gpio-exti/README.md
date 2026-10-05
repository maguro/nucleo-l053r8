# gpio-exti

This example toggles the user LED (LD2 on PA5) each time you press the user button (B1, the blue button). The system clock is MSI at about 2.1 MHz. The program logs `gpio-exti started` at startup.

## Origin

The C example is the STM32CubeL0 `GPIO_EXTI` example for NUCLEO-L053R8 (`Projects/NUCLEO-L053R8/Examples/GPIO/GPIO_EXTI/Src/main.c`).

| C example | Rust port |
|-----------|-----------|
| `HAL_Init()` and `SystemClock_Config()`: MSI range 5, voltage range 3 | `board::init(SystemClock::Msi2MHz)` |
| `BSP_LED_Init(LED2)` | `Led2::new(p.PA5)` |
| `EXTILine4_15_Config()`: PC13, falling edge, EXTI4_15 interrupt | `UserButton::new(p.PC13, p.EXTI13)` |
| `HAL_GPIO_EXTI_Callback()`, which toggles LED2 | the body of the `main()` loop |

In the C example, the interrupt callback toggles LD2. In this port, the interrupt only wakes `main`, and `main` toggles LD2. Thus the interrupt handler stays short, and the work runs in a task.

## Running it

Run these commands from the repository root:

```bash
cargo run -p gpio-exti
cargo run -p gpio-exti --release
```

The dev build uses about 22 KB of flash. The release build uses about 9 KB.

The [blinky README](../blinky/README.md) tells how to stop probe-rs, reconnect with `probe-rs attach`, and debug in RustRover. These steps also apply to gpio-exti. In the commands and paths, use `gpio-exti` in place of `blinky`.
