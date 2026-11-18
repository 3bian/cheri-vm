// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod debug_args;
mod profile_args;
mod run_args;
mod run_command_args;

pub(crate) use debug_args::DebugArgs;
pub(crate) use profile_args::ProfileArgs;
pub(crate) use run_args::RunArgs;
pub(crate) use run_command_args::RunCommandArgs;

use cheri_vm::program::arguments::{
    CommandSpec, Flag, Parsed, Positional, Program,
};

use cheri_vm::program::commands::{APPLICATION, COMMANDS, SUMMARY};

use cheri_vm::program::options::{COLOR, FORMAT, OUTPUT, PROFILE, QUIET};

use cheri_vm::program::output::Format;

const ADDRESS: Flag =
    Flag::value("address", "ADDRESS", HELP_ADDRESS).requiring(&["raw"]);

const BREAKPOINT: Flag =
    Flag::value("breakpoint", "ADDRESS", HELP_BREAKPOINT).repeated();

const COMPLETION: Flag =
    Flag::value("completion", "disabled|elf-symbol|ADDRESS", HELP_COMPLETION);

const CONSOLE_FORMAT: Flag = Flag::choice(
    "console-format",
    &["raw", "labeled", "escaped"],
    HELP_CONSOLE_FORMAT,
);

const DAP: Flag = Flag::switch("dap", HELP_DAP).conflicting(&["gdb"]);

const ETHERNET: Flag = Flag::value(
    "ethernet",
    "interface=NAME|ws|ws://HOST:PORT/PATH[,KEY=VALUE]...",
    HELP_ETHERNET,
)
.repeated();

const FIRMWARE: Positional = Positional {
    help: "Firmware to run: an ELF file, or with --raw a raw image",
    name: "FIRMWARE",
    unless: None,
};

const GDB: Flag = Flag::value("gdb", "PROGRAM", HELP_GDB);

const GDB_PORT: Flag =
    Flag::number("gdb-port", "PORT", 65535, HELP_GDB_PORT).with_default("3333");

const GPIO: Flag = Flag::value(
    "gpio",
    "path=CHIP,lines=N+N|ws|ws://HOST:PORT/PATH[,KEY=VALUE]...",
    HELP_GPIO,
)
.repeated();

const HELP_ADDRESS: &str = "Load address of the raw image";

const HELP_BREAKPOINT: &str =
    "Stop at an address; repeat to add more breakpoints";

const HELP_COMPLETION: &str = "How the guest reports completion";

const HELP_CONSOLE_FORMAT: &str = "How guest console output is shown: raw bytes, prefixed with the \
             UART name, or with control characters escaped";

const HELP_DAP: &str = "Serve the Debug Adapter Protocol on standard input and output, for \
             editors such as VS Code";

const HELP_ETHERNET: &str = "Bind an Ethernet device to a host TAP interface, which Linux \
             provides, or to a WebSocket: ws serves the device at /ws/NAME, and \
             ws://HOST:PORT/PATH connects to a program that is listening. \
             Example: interface=tap0 or ws\nOptional keys: device=NAME (profile \
             device); with a WebSocket, timeout=MS. Defaults to the \
             CHERI_VM_ETHERNET environment variable";

const HELP_GDB: &str = "GDB client to launch against the server";
const HELP_GDB_PORT: &str = "TCP port the GDB server listens on";

const HELP_GPIO: &str = "Bind a GPIO device to host GPIO lines, through the Linux GPIO \
             character device, or to a WebSocket: ws serves the device at \
             /ws/NAME, and ws://HOST:PORT/PATH connects to a program that is \
             listening. Example: path=/dev/gpiochip0,lines=17+27 or \
             ws,pins=5+6\nOptional keys: device=NAME (profile device); with \
             path, active-low=true|false, \
             bias=as-is|disabled|pull-down|pull-up, \
             drive=push-pull|open-drain|open-source; with a WebSocket, pins=N+N \
             (the pins to send; default all) and timeout=MS. Defaults to the \
             CHERI_VM_GPIO environment variable";

const HELP_HOST: &str =
    "Loopback IP address that every server of the run listens on";

const HELP_I2C: &str = "Bind an I2C device to a host bus, through Linux i2c-dev, or to a \
             WebSocket: ws serves each address at /ws/NAME/ADDRESS, and \
             ws://HOST:PORT/PATH connects to a program that is listening. \
             Example: path=/dev/i2c-1,addresses=0x48+0x49 or \
             ws,addresses=0x48\nOptional keys: device=NAME (profile device), \
             addresses=A+A (with path, the host addresses to allow; with a \
             WebSocket, the addresses to send, which it needs); with a \
             WebSocket, timeout=MS. Defaults to the CHERI_VM_I2C environment \
             variable";

const HELP_LIST: &str = "List the names this command takes, and do nothing \
                         else: the built-in profiles, with their descriptions";

const HELP_MEMORY: &str = "Dump memory when the run ends. Example: \
             address=0x80000000,length=64\nlength is 1 to 256 bytes and \
             defaults to 16";

const HELP_RAW: &str = "Read FIRMWARE as a raw image, loaded at --address";

const HELP_REGISTERS: &str = "Print registers when the run ends. Example: pc+x1+x2\nNames: \
             x0..x31, pc, f0..f31 and fcsr with floating point, pcc with RVY, \
             and on CHERIoT c0..c15, pcc, mtcc, mtdc, mscratchc, mepcc";

const HELP_SPI: &str = "Bind an SPI device to a host bus, through Linux spidev, or to a \
             WebSocket: ws serves the chip select at /ws/NAME/SELECT, and \
             ws://HOST:PORT/PATH connects to a program that is listening. \
             Example: path=/dev/spidev0.0,speed=1000000 or \
             ws,write-only=true\nOptional keys: device=NAME (profile device), \
             select=0..3, write-only=true|false (send bursts and read nothing \
             back); with path, speed=HZ (default 500000); with a WebSocket, \
             timeout=MS. Defaults to the CHERI_VM_SPI environment variable";

const HELP_STATS: &str = "Report instructions and code coverage per compartment, the hottest \
             functions, and stack high-water marks; --format and -o say how";

const HELP_STEPS: &str = "Stop after N steps; 0 means no limit";
const HELP_TRACE: &str = "Print every executed instruction";
const HELP_TRAP_POLICY: &str = "Stop on a trap, or vector to the guest handler";

const HELP_UART: &str = "Bind a UART to a host file, pipe or TTY, or to a WebSocket, which \
             carries both directions: ws serves the UART at /ws/NAME, and \
             ws://HOST:PORT/PATH connects to a program that is listening. \
             Example: out=/dev/stdout or \
             in=/dev/ttyUSB0,out=/dev/ttyUSB0,baud=115200 or ws\nOptional keys: \
             device=NAME (profile device); with in or out, baud=N, data-bits=N, \
             stop-bits=N, parity=none|even|odd, flow=none|rtscts|xonxoff; with \
             a WebSocket, timeout=MS. Defaults to the CHERI_VM_UART environment \
             variable";

const HELP_WEB_PORT: &str = "TCP port of the web server, which serves the pages of --web-root \
             and the devices bound with ws under /ws/; 0 takes a free port";

const HELP_WEB_ROOT: &str = "Folder of pages to serve, subfolders included; / is its \
             index.html. Without it only /ws/ is served";

const HOST: Flag =
    Flag::value("host", "ADDRESS", HELP_HOST).with_default("127.0.0.1");

const I2C: Flag = Flag::value(
    "i2c",
    "path=BUS|ws|ws://HOST:PORT/PATH[,KEY=VALUE]...",
    HELP_I2C,
)
.repeated();

const LIST: Flag = Flag::switch("list", HELP_LIST);

const MEMORY: Flag =
    Flag::value("memory", "address=ADDRESS[,length=N]", HELP_MEMORY);

pub(crate) static PROGRAM: Program = Program {
    commands: &[
        CommandSpec {
            name: COMMANDS[0].name,
            options: &[
                ADDRESS,
                BREAKPOINT,
                COMPLETION,
                CONSOLE_FORMAT,
                DAP,
                ETHERNET,
                STATS_FORMAT,
                GDB,
                GDB_PORT,
                GPIO,
                HOST,
                I2C,
                MEMORY,
                STATS_OUTPUT,
                PROFILE,
                RAW,
                REGISTERS,
                SPI,
                STATS,
                STEPS,
                TRACE,
                TRAP_POLICY,
                UART,
                WEB_PORT,
                WEB_ROOT,
            ],
            positionals: &[FIRMWARE],
            summary: COMMANDS[0].summary,
        },
        CommandSpec {
            name: COMMANDS[1].name,
            options: &[FORMAT, LIST, OUTPUT],
            positionals: &[Positional {
                help: "Built-in profile to print, as TOML",
                name: "NAME",
                unless: Some("list"),
            }],
            summary: COMMANDS[1].summary,
        },
        CommandSpec {
            name: COMMANDS[2].name,
            options: &[
                ADDRESS,
                BREAKPOINT,
                COMPLETION,
                CONSOLE_FORMAT,
                ETHERNET,
                STATS_FORMAT,
                GPIO,
                HOST,
                I2C,
                MEMORY,
                STATS_OUTPUT,
                PROFILE,
                QUIET,
                RAW,
                REGISTERS,
                SPI,
                STATS,
                STEPS,
                TRACE,
                TRAP_POLICY,
                UART,
                WEB_PORT,
                WEB_ROOT,
            ],
            positionals: &[FIRMWARE],
            summary: COMMANDS[2].summary,
        },
    ],
    name: APPLICATION,
    options: &[COLOR],
    summary: SUMMARY,
    version: cheri_vm::program::VERSION,
};

const RAW: Flag = Flag::switch("raw", HELP_RAW).requiring(&["address"]);

const REGISTERS: Flag =
    Flag::value("registers", "NAME+NAME...", HELP_REGISTERS);

const SPI: Flag = Flag::value(
    "spi",
    "path=BUS|ws|ws://HOST:PORT/PATH[,KEY=VALUE]...",
    HELP_SPI,
)
.repeated();

const STATS: Flag = Flag::switch("stats", HELP_STATS);
const STATS_FORMAT: Flag = FORMAT.requiring(&["stats"]);

pub(crate) const STATS_FORMATS: &[Format] =
    &[Format::Text, Format::Json, Format::Toml];

const STATS_OUTPUT: Flag = OUTPUT.requiring(&["stats"]);
const STEPS: Flag = Flag::number("steps", "N", u64::MAX, HELP_STEPS);
const TRACE: Flag = Flag::switch("trace", HELP_TRACE);

const TRAP_POLICY: Flag =
    Flag::choice("trap-policy", &["stop", "vector"], HELP_TRAP_POLICY);

const UART: Flag = Flag::value(
    "uart",
    "in=PATH|out=PATH|ws|ws://HOST:PORT/PATH[,KEY=VALUE]...",
    HELP_UART,
)
.repeated();

const WEB_PORT: Flag =
    Flag::number("web-port", "PORT", 65535, HELP_WEB_PORT).with_default("8080");

const WEB_ROOT: Flag = Flag::value("web-root", "DIR", HELP_WEB_ROOT);

fn port(parsed: &Parsed, long: &str) -> u16 {
    parsed
        .number(long)
        .and_then(|port| u16::try_from(port).ok())
        .unwrap_or_default()
}
