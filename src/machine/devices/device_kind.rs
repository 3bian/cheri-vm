// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{DEVICE_KINDS, Device, DeviceKey};
use crate::machine::configuration::DeviceConfiguration;
use crate::machine::error::EmulatorError;

pub(crate) struct DeviceKind {
    pub(crate) name: &'static str,
    pub(crate) size: u32,
    pub(crate) keys: &'static [DeviceKey],
    pub(crate) needs_cheriot: bool,
    pub(crate) controls_interrupts: bool,
    pub(crate) routes_interrupts: bool,
    pub(crate) make:
        fn(&DeviceConfiguration) -> Result<Box<dyn Device>, EmulatorError>,
}

impl DeviceKind {
    pub(crate) fn backing(
        &self,
        device: &DeviceConfiguration,
    ) -> Option<usize> {
        if self.takes(DeviceKey::Storage) {
            device.sectors.checked_mul(512)
        } else {
            Some(0)
        }
    }

    pub(crate) fn check_keys(
        &self,
        attachment: bool,
        storage: bool,
        layout: bool,
    ) -> Result<(), EmulatorError> {
        for (given, key, words) in [
            (
                attachment,
                DeviceKey::AttachedDevice,
                "attached_device is unsupported for",
            ),
            (storage, DeviceKey::Storage, "storage parameters given for"),
            (layout, DeviceKey::Layout, "UART layout is unsupported for"),
        ] {
            if given && !self.takes(key) {
                return Err(EmulatorError::Configuration(format!(
                    "{words} {}",
                    self.name
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn named(name: &str) -> Result<&'static Self, EmulatorError> {
        DEVICE_KINDS
            .iter()
            .find(|kind| kind.name == name)
            .ok_or_else(|| {
                let names: Vec<&str> =
                    DEVICE_KINDS.iter().map(|kind| kind.name).collect();
                EmulatorError::Configuration(format!(
                    "unsupported device type: {name}; the types are {}",
                    names.join(", ")
                ))
            })
    }

    pub(crate) fn takes(&self, key: DeviceKey) -> bool {
        self.keys.contains(&key)
    }
}
