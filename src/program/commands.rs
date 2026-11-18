// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub const APPLICATION: &str = "cheri-vm";

pub const COMMANDS: &[Command] = &[
    command(
        "debug",
        "Debug firmware in the emulator, through a GDB server by default",
    ),
    command(
        "profile",
        "Print a profile built into the emulator, or list them",
    ),
    command("run", "Run firmware in the emulator"),
];

pub const ETHERNET_VARIABLE: &str = "CHERI_VM_ETHERNET";
pub const GPIO_VARIABLE: &str = "CHERI_VM_GPIO";
pub const I2C_VARIABLE: &str = "CHERI_VM_I2C";
pub const PROFILE_VARIABLE: &str = "CHERI_VM_PROFILE";
pub const SPI_VARIABLE: &str = "CHERI_VM_SPI";

pub const SUMMARY: &str = "Run and debug firmware in the emulator: CHERIoT, \
                           RV32, RV64, CVA6 and RVY machines. It takes a \
                           firmware file and a profile, and builds nothing";

pub const UART_FROM_STANDARD_INPUT: &str = "in=/dev/stdin";
pub const UART_VARIABLE: &str = "CHERI_VM_UART";

pub struct Command {
    pub name: &'static str,
    pub summary: &'static str,
}

const fn command(name: &'static str, summary: &'static str) -> Command {
    Command { name, summary }
}

pub fn names(
    commands: &'static [Command],
) -> impl Iterator<Item = &'static str> {
    commands.iter().map(|command| command.name)
}

pub fn summary(commands: &[Command], name: &str) -> &'static str {
    commands
        .iter()
        .find(|command| command.name == name)
        .map_or("", |command| command.summary)
}
