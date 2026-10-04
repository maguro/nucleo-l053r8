# AGENTS.md

This file contains directives for coding agents that work in this repository.

## Purpose

This repository is an example project for programming the STMicroelectronics NUCLEO-L053R8 board in Rust. It has three goals.

1. Provide Rust ports of the existing C examples for this board, such as those in ST's STM32CubeL0 package.
2. Serve as a tutorial that shows how to set up Rust embedded development for this board.
3. Keep the setup guidance general enough that it also applies to other STM32 boards.

Code and docs are teaching material. Favor clarity and explanation over cleverness. Each example should be readable on its own by someone who knows the matching C example.

## Writing style

All documentation and code comments must follow ASD-STE100 (Simplified Technical English). This rule applies to new text and to each change to existing text. It covers Markdown files, Rust doc comments, Rust code comments, and TOML comments.

`.claude/skills/asd-ste100/SKILL.md` contains the rules and the review process. Claude Code loads this file as the `asd-ste100` skill. Other agents must read this file before they write documentation or comments.

## Layout

- `Cargo.toml` is a virtual workspace. The members are the `board` crate and every directory under `examples/`. Each example crate holds one ported example, and its binary has the same name as the crate.
- `board/` is a library crate that does the same job as ST's board support package (BSP). `board` sets up the system clock, the user LED (LD2), and the user button (B1). Examples use `board` and do not set up the clock, the LED, or the button themselves. Put new board-level support in `board`, not in an example.
- `board` binds the `EXTI4_15` interrupt. An example that uses `board` must not bind `EXTI4_15` again.
- The root `Cargo.toml` pins dependency versions and features once, in `[workspace.dependencies]`. Example crates opt in with `foo.workspace = true`. They must not set their own versions.
- Only the root `Cargo.toml` defines build profiles. The dev profile builds the workspace crates (`board` and the examples) at `opt-level = 0` for debugging. It builds all dependencies at `opt-level = "s"`, because unoptimized embassy code does not fit comfortably in 64 KB. The release profile uses `opt-level = "s"` with LTO.
- `.cargo/config.toml` sets the default target, the linker scripts (`link.x` from cortex-m-rt, `defmt.x` from defmt), the probe-rs runner, and `DEFMT_LOG`.
- There is no hand-written `memory.x`. The `memory-x` feature of embassy-stm32 generates it from the chip feature `stm32l053r8`.

## HAL and runtime

Examples use embassy: `embassy-stm32` for peripherals, `embassy-executor` for async tasks, and `embassy-time` for delays. The time driver comes from `time-driver-any`, which reserves one hardware timer. Logging uses `defmt` over RTT. `panic-probe` prints panic messages. `embassy_stm32::init(Default::default())` runs the core from MSI at about 4.2 MHz, not the chip's 2.1 MHz reset clock. Examples that need a different clock set it explicitly in `embassy_stm32::Config`.

Each example's top doc comment names the C example that it ports.

## Commands

README.md holds the user-facing one-time setup and board connection steps. Update README.md when setup requirements change. Keep that content out of this file. Do not add troubleshooting sections to README.md.

Run all of these commands from the repository root.

```bash
cargo build                    # all examples
cargo build -p blinky          # one example
cargo run -p blinky            # flash over ST-LINK and stream defmt logs
cargo run -p blinky --release
cargo clippy
cargo fmt --check
```

There are no host-side tests. To verify code, build it and run it on the board.

## Target hardware

- MCU: STM32L053R8T6, Arm Cortex-M0+, 32 MHz max.
- Memory: 64 KB flash at `0x0800_0000`, 8 KB SRAM at `0x2000_0000`, 2 KB data EEPROM.
- Rust target triple: `thumbv6m-none-eabi`. Cortex-M0+ has no hardware divide and no atomic compare-and-swap, so crates that need CAS atomics require `portable-atomic` or `critical-section`.
- On-board debugger: ST-LINK/V2-1, which also exposes a virtual COM port on USART2 (PA2 TX, PA3 RX).
- User LED LD2 on PA5. User button B1 on PC13.

The 8 KB of RAM is the tightest constraint. When you add an example, check the stack and static sizes. Do not use heap allocation unless the example is specifically about it.
