// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod ethernet;
pub(crate) mod gpio;
pub(crate) mod host;
pub(crate) mod i2c;
pub(crate) mod run;
pub(crate) mod socket;
pub(crate) mod spi;
pub(crate) mod uart;

use crate::hart::execution::TrapPolicy;
use crate::machine::completion::CompletionPolicy;

use crate::runtime::output::OutputPresentation;
use crate::runtime::register_selection::RegisterSelection;

use super::options::host::HostOptions;
use crate::machine::EmulatorError;

pub(crate) struct Options {
    pub breakpoint: Vec<u64>,
    pub completion: Option<CompletionPolicy>,
    pub firmware_is_elf: bool,
    pub firmware_path: String,
    pub gdb: Option<std::net::SocketAddr>,
    pub gdb_client: Option<std::path::PathBuf>,
    pub gdb_raw_output: bool,
    pub host: HostOptions,
    pub memory_dump: Option<(u64, usize)>,
    pub output: Option<OutputPresentation>,
    pub profile_path: String,
    pub profile_text: Option<String>,
    pub stats: Option<crate::runtime::request::StatsReport>,
    pub raw_address: u64,
    pub exit_line: crate::runtime::request::ExitLine,
    pub registers: Option<RegisterSelection>,
    pub steps: Option<u64>,
    pub trace: bool,
    pub trap_policy: Option<TrapPolicy>,
}

pub(super) fn parse_u32(value: &str, name: &str) -> Result<u32, EmulatorError> {
    u32::try_from(parse_unsigned(value, name)?).map_err(|_| {
        EmulatorError::Configuration(format!("{name} exceeds 32 bits"))
    })
}

pub(super) fn parse_unsigned(
    value: &str,
    name: &str,
) -> Result<u64, EmulatorError> {
    let parsed = match value.strip_prefix("0x") {
        Some(hexadecimal) => u64::from_str_radix(hexadecimal, 16),
        None => value.parse(),
    };
    parsed.map_err(|_| {
        EmulatorError::Configuration(format!("{name} is not a number: {value}"))
    })
}
