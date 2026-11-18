// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod breakpoint_type;
pub(crate) mod core;
mod hart;
pub(crate) mod run_control;
mod run_state;
mod session_target;
mod stop_reason;
pub(crate) mod target;

pub(crate) use self::core::{Core, GdbArchitecture};
pub(crate) use breakpoint_type::BreakpointType;
pub(crate) use run_control::RunControl;
pub(crate) use run_state::RunState;
pub(crate) use session_target::SessionTarget;
pub(crate) use stop_reason::StopReason;
pub(crate) use target::Target;

pub(crate) fn monitor_words(command: &[u8]) -> Result<Vec<&str>, &'static str> {
    if command.len() > 128 {
        return Err("monitor command too long");
    }
    let command =
        std::str::from_utf8(command).map_err(|_| "monitor requires UTF-8")?;
    Ok(command.split_whitespace().collect())
}

pub(crate) fn read_register(
    bytes: &[Option<u8>],
    output: &mut [u8],
) -> Option<usize> {
    if output.len() < bytes.len() {
        return None;
    }
    if bytes.iter().any(Option::is_none) {
        return Some(0);
    }
    for (destination, value) in output.iter_mut().zip(bytes) {
        *destination = (*value)?;
    }
    Some(bytes.len())
}
