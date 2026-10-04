# nucleo-l053r8
This is a Rust project for the STMicroelectronics STM32 Nucleo-64 MCU Development Board, NUCLEO-L053R8.

## One-time setup

Do these steps once per computer.

1. Install Rust from [rustup.rs](https://rustup.rs). Then add the Cortex-M0+ target:
   ```bash
   rustup target add thumbv6m-none-eabi
   ```
2. Install [probe-rs](https://probe.rs), which flashes the board and shows log output:
   ```bash
   cargo install probe-rs-tools --locked
   ```
3. Check the board jumpers. The factory defaults are correct (UM1724, section 5.1):
   - CN2: both jumpers are on, so the on-board ST-LINK programs the MCU of this board.
   - JP1: the jumper is off.
   - JP5 (labeled PWR): the jumper is on U5V, so the board gets power from USB.
   - JP6 (labeled IDD): the jumper is on, so the MCU gets power.
4. Connect the board with a Mini-B USB cable to the ST-LINK connector, CN1, at the top of the board. The red PWR LED and the COM LED (LD1) turn on. macOS needs no driver. Linux needs the udev rules from the [probe-rs setup guide](https://probe.rs/docs/getting-started/probe-setup/).
5. Confirm that probe-rs sees the board:
   ```bash
   probe-rs list
   ```
   probe-rs should list one probe, for example:
   ```
   [0]: STLink V2-1 -- 0483:374b:0671FF575383564867211750 (ST-LINK)
   ```
   Run the command two or three times. probe-rs should list the probe every time. If probe-rs reports outdated ST-LINK firmware, update the firmware with ST's [STSW-LINK007](https://www.st.com/en/development-tools/stsw-link007.html) tool.

## Running an example

When the board is connected, run any example by its name from the repository root:

```bash
cargo run -p blinky
```

This command builds the example, writes it to flash, resets the board, and streams its log output. Press Ctrl+C to stop the log stream. The program stays in flash and continues to run.

Each directory under `examples/` is one example.
