// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub mod adapter;
pub(crate) mod architecture;
#[cfg(feature = "gdb")]
pub(crate) mod gdb;

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;

pub(crate) fn inspect_memory(
    bus: &Bus,
    physical: impl Fn(u64) -> Option<u64>,
    address: u64,
    data: &mut [u8],
) -> Result<(), EmulatorError> {
    let outside = || {
        EmulatorError::Configuration(
            "debug memory query exceeds its range or 1024-byte budget".into(),
        )
    };
    if data.len() > 1024 {
        return Err(outside());
    }
    for (offset, byte) in data.iter_mut().enumerate() {
        let location =
            address.checked_add(offset as u64).ok_or_else(outside)?;
        let location = physical(location).ok_or(EmulatorError::Bus {
            address: location,
            explanation: "inspection outside mapped memory".into(),
        })?;
        *byte = bus.inspect(location, 1)? as u8;
    }
    Ok(())
}
