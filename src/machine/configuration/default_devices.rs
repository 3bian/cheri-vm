// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::ethernet::{Ethernet, NetworkBackend};
use crate::machine::devices::gpio::Gpio;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefaultDevices {
    CheriotCompatibility,
    Configured,
}

impl DefaultDevices {
    pub(crate) fn install(self, bus: &mut Bus) -> Result<(), EmulatorError> {
        if self == Self::Configured {
            return Ok(());
        }
        if !bus
            .devices()
            .iter()
            .any(|mapping| mapping.device.is::<Ethernet>())
        {
            bus.add_device(
                "ethernet".into(),
                0x2000_0000,
                0x100,
                Box::new(Ethernet::new(NetworkBackend::InMemory)),
            )?;
        }
        if !bus
            .devices()
            .iter()
            .any(|mapping| mapping.device.is::<Gpio>())
        {
            bus.add_device(
                "gpio".into(),
                0x1000_3000,
                0x100,
                Box::new(Gpio::new()),
            )?;
        }
        if bus.devices().len() > 64 {
            return Err(EmulatorError::Configuration(
                "at most 64 devices, including compatibility defaults, are \
                 supported"
                    .into(),
            ));
        }
        Ok(())
    }
}
