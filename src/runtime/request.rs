// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod address_kind;
mod exit_line;
mod firmware;
mod machine_request;
mod peripheral_settings;
mod presentation;
mod register_dump;
mod stats_report;
mod web_settings;

pub use address_kind::AddressKind;
pub use exit_line::ExitLine;
pub use firmware::Firmware;
pub use machine_request::MachineRequest;
pub use peripheral_settings::PeripheralSettings;
pub use presentation::Presentation;
pub use register_dump::RegisterDump;
pub use stats_report::StatsReport;
pub use web_settings::WebSettings;

use super::options::parse_unsigned;

use crate::machine::EmulatorError;

pub fn address(value: &str, kind: AddressKind) -> Result<u64, EmulatorError> {
    let name = match kind {
        AddressKind::Raw => "raw address",
        AddressKind::Breakpoint => "breakpoint address",
        AddressKind::Completion => "completion address",
    };
    parse_unsigned(value, name)
}
