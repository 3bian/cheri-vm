// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{
    ADDRESS, BREAKPOINT, COMPLETION, CONSOLE_FORMAT, ETHERNET, GPIO, HOST, I2C,
    MEMORY, REGISTERS, SPI, STATS, STEPS, TRACE, TRAP_POLICY, UART, WEB_PORT,
    WEB_ROOT, port,
};
use cheri_vm::program::arguments::{Flag, Parsed};
use cheri_vm::program::options::{FORMAT, OUTPUT, PROFILE};
use cheri_vm::program::output::Format;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct RunArgs {
    pub address: Option<String>,
    pub breakpoint: Vec<String>,
    pub completion: Option<String>,
    pub console_format: Option<String>,
    pub ethernet: Vec<String>,
    pub file: Option<PathBuf>,
    pub format: Option<Format>,
    pub gpio: Vec<String>,
    pub host: String,
    pub i2c: Vec<String>,
    pub memory: Option<String>,
    pub output: Option<PathBuf>,
    pub profile: Option<String>,
    pub registers: Option<String>,
    pub spi: Vec<String>,
    pub stats: bool,
    pub steps: Option<u64>,
    pub trace: bool,
    pub trap_policy: Option<String>,
    pub uart: Vec<String>,
    pub web_port: u16,
    pub web_root: Option<PathBuf>,
}

impl RunArgs {
    pub(crate) fn from(parsed: &Parsed) -> Self {
        let text = |flag: &Flag| parsed.one(flag.long).map(str::to_owned);
        Self {
            address: text(&ADDRESS),
            breakpoint: parsed.all(BREAKPOINT.long),
            completion: text(&COMPLETION),
            console_format: text(&CONSOLE_FORMAT),
            ethernet: parsed.all(ETHERNET.long),
            file: parsed.positionals.first().map(PathBuf::from),
            format: parsed.one(FORMAT.long).and_then(Format::named),
            gpio: parsed.all(GPIO.long),
            host: text(&HOST).unwrap_or_default(),
            i2c: parsed.all(I2C.long),
            memory: text(&MEMORY),
            output: parsed.one(OUTPUT.long).map(PathBuf::from),
            profile: text(&PROFILE),
            registers: text(&REGISTERS),
            spi: parsed.all(SPI.long),
            stats: parsed.has(STATS.long),
            steps: parsed.number(STEPS.long),
            trace: parsed.has(TRACE.long),
            trap_policy: text(&TRAP_POLICY),
            uart: parsed.all(UART.long),
            web_port: port(parsed, WEB_PORT.long),
            web_root: parsed.one(WEB_ROOT.long).map(PathBuf::from),
        }
    }
}
