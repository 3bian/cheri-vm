# cheri-vm

The CHERI and RISC-V emulator: CHERIoT, RV32, RV64, CVA6 and RVY machines.

## Command Line Options

```text
Usage: cheri-vm [OPTIONS] [COMMAND]

Run and debug firmware in the emulator: CHERIoT, RV32, RV64, CVA6 and RVY
machines. It takes a firmware file and a profile, and builds nothing.

Commands:
  debug
    Debug firmware in the emulator, through a GDB server by default.

  profile
    Print a profile built into the emulator, or list them.

  run
    Run firmware in the emulator.

Options:
  --color auto|always|never
    Color text output: auto (only on a terminal, and not when NO_COLOR is set),
    always or never. Defaults to the color setting, else auto.

  -V, --version
    Display the version, and exit.

  -h, --help
    Display this help and exit.
```

Each command prints its own options with `--help`, as in
`cheri-vm run --help`. Addresses and counts accept decimal or `0x`
hexadecimal values. Repeat a device option to connect different devices.

## About

`cheri-vm` runs and debugs firmware on an emulated machine, with simulated
memory and devices. It is given two things: a firmware file, which is an ELF
file or a raw image, and a profile, which describes the machine. It builds
nothing and reads no project.

Seven machines are built in, and `cheri-vm profile --list` names them:

- `rv32e` and `rv32imc`, two bare 32-bit machines.
- `rv64imac`, a bare 64-bit machine.
- `cva6`, the CVA6 processor as software sees it, which runs OpenSBI and
  Linux.
- `cva6-cheri`, the same with RVY capabilities.
- `cheriot` and `cheriot-hwr`, the CHERIoT machine, without and with its
  hardware revoker in use.

A profile of your own is a TOML file, which may extend one of these.

## Build and Run

The emulator is written in Rust and needs nothing else to build. Install
Rust with rustup, then install the emulator from this folder:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
  sh -s -- -y --profile minimal
. "$HOME/.cargo/env"

cargo install --locked --path .
```

`examples/` holds small programs for each machine, as source and as built
images, so the emulator can run without building a guest:

```sh
cheri-vm run --profile rv32imc --raw examples/assembly/rv32/hello-uart.bin --address 0x80000000
```

The program writes `A` to the UART and stops at a breakpoint.

An ELF file says where it is loaded and where it starts. A raw image does
not, so `--raw` needs `--address`:

```sh
cheri-vm run firmware.elf --profile cheriot
cheri-vm run --raw firmware.bin --address 0x80000000 --profile rv32imc --steps 100000
```

`--trace`, `--registers`, `--memory` and `--stats` report what a run did:

```sh
cheri-vm run firmware.elf --profile cheriot --stats -o stats.toml
```

## Debugging

`cheri-vm debug` starts a GDB server, which GDB and LLDB connect to, and
with `--dap` it serves an editor through the Debug Adapter Protocol:

```sh
cheri-vm debug firmware.elf --profile cheriot --gdb-port 3333
cheri-vm debug firmware.elf --profile cheriot --gdb riscv32-unknown-elf-gdb
```

## Devices

A device option connects a device of the machine to the host: a file, a
terminal or a Linux device, or a WebSocket.

```sh
cheri-vm run firmware.elf --profile cheriot --uart in=/dev/stdin
cheri-vm run firmware.elf --profile cheriot --uart in=/dev/ttyUSB0,out=/dev/ttyUSB0,baud=115200
cheri-vm run firmware.elf --profile rv32imc --gpio path=/dev/gpiochip0,lines=17+27
```

`ws` serves the device from the emulator's own web server, at an address
under `/ws/`, and `--web-root` serves a folder of pages beside it. The run
waits for each such device to be connected:

```sh
cheri-vm run --profile rv32imc --raw examples/assembly/rv32/echo-uart.bin --address 0x80000000 --uart ws --web-root examples/websocket
cheri-vm run --profile rv32imc --raw examples/assembly/rv32/i2c-sensor.bin --address 0x80000000 --i2c ws,addresses=0x48 --web-root examples/websocket
cheri-vm run --profile rv32imc --raw examples/assembly/rv32/lcd.bin --address 0x80000000 --spi ws,write-only=true --gpio ws,pins=0+1 --web-root examples/websocket
```

Then open `http://127.0.0.1:8080/` in a browser on the same computer.
`ws://HOST:PORT/PATH` connects the device to a program that is listening
instead:

```sh
cheri-vm run firmware.elf --profile cheriot --uart ws://127.0.0.1:9000/uart
```

The web server listens on a loopback address only and has no password:
another user of the same computer can connect to a device.

## The Emulation Loop

One loop runs every machine, and each turn of it is one guest instruction.

- `run_to_stop`, in `src/runtime/run.rs`, is the loop. It checks for a
  breakpoint or the step limit, calls `step`, and stops on an exit, a fatal
  trap or the firmware's completion.
- `Hart::step`, in `src/hart/step.rs`, is one instruction. It brings the
  timers up to date, then takes a pending interrupt or fetches, decodes and
  executes, and enters the trap handler on a fault.
- `Hart::fetch`, in the same file, checks the program counter, reads 16 or
  32 bits, and decodes them through `src/isa/decode.rs`, behind a cache.
- `Architecture::execute`, in `src/hart/plain/execute.rs` and
  `src/hart/cheriot_rv32e/execute.rs`, carries out the decoded instruction
  on the registers and the bus.
- `Bus::read` and `Bus::write`, in `src/machine/bus.rs`, are a load or a
  store, sent to memory or to the device at that address.
- `Architecture::enter_trap`, in `src/hart/plain.rs` and
  `src/hart/cheriot_rv32e.rs`, saves the cause and the address, and moves
  to the trap vector.
- `poll_host`, in `src/runtime/run.rs`, runs every 1,024 steps and when the
  guest waits. It passes input to the devices and writes their output.

## Folders

- `examples/` holds guests for each machine, as source and as built images,
  pages that play a device over a WebSocket, and two profiles that extend a
  built-in one.
- `integrations/` holds the Visual Studio Code extension.
- `profiles/` holds the machines built into the program.
- `src/` holds the library: the instruction sets, the processors, the
  machine and its devices, the run loop, the debuggers and the web server.
  `src/bin/cheri-vm/` is the program.
- `tests/` holds the unit tests in `library/` and `binary/`, the end-to-end
  suite in `cli/`, and their `fixtures/`, `support/` and help `snapshots/`.

## License

CHERI and CHERIoT profiles are experimental; complete compliance and security
isolation have not been established. Licensed under the [MIT license](LICENSE).
