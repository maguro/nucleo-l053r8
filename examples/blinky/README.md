# blinky

This example toggles the user LED (LD2 on PA5) every 500 ms. It logs `blinky started` once, at startup.

It is the Rust counterpart of the STM32CubeL0 `GPIO_IOToggle` example.

## Running it

Run this command from the repository root:

```bash
cargo run -p blinky
```

`cargo run` normally starts the built program on your computer. This repository's `.cargo/config.toml` sets a runner for the `thumbv6m-none-eabi` target, so Cargo instead runs:

```
probe-rs run --chip STM32L053R8Tx --no-catch-reset target/thumbv6m-none-eabi/debug/blinky
```

The `--no-catch-reset` flag is important. Without this flag, probe-rs sets the reset vector catch of the core. The reset vector catch is a debug setting that halts the core immediately after any reset. probe-rs does not clear this setting when it exits. The setting stays set after a press of the RESET button, and only a power cycle of the board clears it. As a result, until you power-cycle the board, each press of RESET halts the program.

probe-rs writes the program to flash, resets the chip, and stays attached to print the program's log messages. The messages come from defmt macros such as `info!`. The program writes them to a 1 KB buffer in RAM, and probe-rs reads that buffer over the debug connection. The log messages do not go through the USB serial port of the board. `DEFMT_LOG` in `.cargo/config.toml` sets the log levels that the compiler includes. Its value is `info` by default, so the compiler leaves out `debug!` and `trace!` messages.

## Running an optimized build

```bash
cargo run -p blinky --release
```

This command uses the release profile in the root `Cargo.toml`. That profile optimizes all code for size and uses link-time optimization. The result is much smaller. The release build of blinky uses about 8 KB of flash, and the dev build uses about 20 KB. Both builds keep the debug info. The debug info stays on the host and uses no flash.

Cargo writes the release build to `target/thumbv6m-none-eabi/release/blinky`. Use that path in place of `debug/blinky` when you reconnect or debug.

## Stopping and reconnecting

Ctrl+C stops probe-rs, not the program. The program stays in flash and continues to run. It starts again each time the board gets power or you press RESET (B2).

To see the logs again without a reflash or a reset, run this command:

```bash
probe-rs attach --chip STM32L053R8Tx --no-catch-reset target/thumbv6m-none-eabi/debug/blinky
```

`probe-rs attach` sets the reset vector catch the same way `probe-rs run` does, so pass `--no-catch-reset` here too. probe-rs uses the ELF file to find the log buffer and to decode messages. Because of this, pass the same build that is on the board. Pass `debug/` after `cargo run`, and pass `release/` after `cargo run --release`. If you rebuild after you flash the board, the file no longer matches the chip. Messages that the program logs while no debugger is attached stay in the buffer until the buffer is full. After that, the program drops new messages.

After `cargo run` prints `blinky started`, blinky logs nothing more, so `probe-rs attach` shows no new messages for blinky.

## Debugging in RustRover

RustRover has no probe-rs integration. Instead, probe-rs runs a GDB server. RustRover connects to that server with GDB.

1. Use `cargo run -p blinky` to flash the build that you want to debug. Then press Ctrl+C. The GDB server does not flash the chip.
2. Start the GDB server in a terminal. Let the server continue to run:
   ```bash
   probe-rs gdb --chip STM32L053R8Tx
   ```
3. In RustRover, choose Run → Edit Configurations. Click +. Add a **Remote Debug** configuration with these settings:
   - **Debugger:** Custom GDB executable. Use the GDB that comes with RustRover. If you installed RustRover with JetBrains Toolbox on an Apple Silicon Mac, this GDB is at `~/Applications/RustRover.app/Contents/bin/gdb/mac/aarch64/bin/gdb`. This GDB supports Arm targets.
   - **'target remote' args:** `localhost:1337`
   - **Symbol file:** the full path to `target/thumbv6m-none-eabi/debug/blinky`.
   - Leave Sysroot and Path mappings empty.
4. Set breakpoints in the gutter. Start the configuration with Debug. The chip halts when GDB connects. Click Resume to run the program to your breakpoints.

Note these points:

- Only one program can use the ST-LINK at a time. Before you use `cargo run` or `probe-rs attach`, stop the GDB server with Ctrl+C. Before you start the GDB server, stop `cargo run` and `probe-rs attach`.
- You cannot see defmt logs during a debug session.
- Breakpoints in flash use the hardware breakpoint unit of the Cortex-M0+. This unit has four comparators, so you can set a maximum of four breakpoints at one time.
- When you rebuild, also reflash. A symbol file that does not match the flash gives wrong breakpoints and source lines.
- Dev builds compile this crate without optimization, so you can inspect all local variables. Release builds are optimized. In a release build, many variables show as `<optimized out>`.
- When you end the debug session, the core stays halted, so LD2 stops. After you stop the GDB server, restart the program with `probe-rs reset --chip STM32L053R8Tx`. You can also disconnect the board and connect it again.
