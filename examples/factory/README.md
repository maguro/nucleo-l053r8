# factory

This example reproduces the demo that ST installs on the board at the factory. The user LED (LD2 on PA5) toggles at one of three speeds. Each press of the user button (B1, the blue button) selects the next speed:

| Speed | LD2 toggles every |
|-------|-------------------|
| 0 (at startup) | 500 ms |
| 1 | 100 ms |
| 2 | 50 ms |

After speed 2, the next press returns to speed 0. The system clock runs at 32 MHz.

The program logs `factory demo started` at startup. On each press, it logs a line such as `blink speed 1: toggle every 100 ms`.

## Origin

The factory demo is the STM32CubeL0 `Demonstrations` project for NUCLEO-L053R8 (`Projects/NUCLEO-L053R8/Demonstrations/Src/main.c`). This port covers only the no-shield path of the demo. The C demo reads PB0 to detect an Adafruit 1.8" TFT shield. If the shield is attached, the C demo shows a menu and images from the SD card of the shield. In that case, the C demo does not run the no-shield path. This port does not include the shield path.

| C demo | Rust port |
|--------|-----------|
| `HAL_Init()` and `SystemClock_Config()`: HSI16 through the PLL, x4 / 2 | `board::init(SystemClock::Pll32MHz)` |
| `BSP_LED_Init(LED2)` | `Led2::new(p.PA5)` |
| `BSP_PB_Init(BUTTON_KEY, BUTTON_MODE_EXTI)` | `UserButton::new(p.PC13, p.EXTI13)` |
| `LED2_Blink()` loop | `main()` loop |
| `HAL_GPIO_EXTI_Callback()` on the button interrupt | `button_task()`, which sends a `ButtonPress` message to `main()` for each press |
| `BlinkSpeed` global | `speed`, a local variable in `main()` |
| `HAL_Delay()`, which busy-waits | `Timer::after_millis().await`, which lets the core sleep until the timer expires |

The C demo calls ST's board support package (BSP) for the LED and the button. This port uses the [`board`](../../board/src/lib.rs) crate for the same job.

## Button presses

In the C demo, the interrupt callback writes the `BlinkSpeed` global, and the main loop reads it. This sharing is safe only while one piece of code writes `BlinkSpeed`. Nothing in the code enforces this condition.

This port does not share the blink speed. `main` keeps the speed in the local variable `speed`, so no other code can change `speed`. `button_task` waits for a press of B1. Then `button_task` sends a `ButtonPress` message to `main` through the `PRESSES` channel. After each delay, `main` takes all the messages from `PRESSES`. For each message, `main` selects the next speed.

`PRESSES` is a `Channel` that holds up to four presses. Each press is a separate message. Because of this, after two presses during one delay, `main` selects the next speed two times. This behavior is the same as in the C demo. An embassy `Signal` holds only the last value, so a `Signal` would count the two presses as one press.

## Running it

Run these commands from the repository root:

```bash
cargo run -p factory
cargo run -p factory --release
```

The dev build uses about 29 KB of flash. The release build uses about 11 KB.

The [blinky README](../blinky/README.md) tells how to stop probe-rs, reconnect with `probe-rs attach`, and debug in RustRover. These steps also apply to factory. In the commands and paths, use `factory` in place of `blinky`.
