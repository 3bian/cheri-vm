// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod device;
mod device_key;
mod device_kind;

pub(crate) use device::Device;
pub(crate) use device_key::DeviceKey;
pub(crate) use device_kind::DeviceKind;

use crate::hart::cheriot_rv32e::Processor as CheriotProcessor;

use crate::machine::configuration::DeviceConfiguration;

use crate::machine::error::EmulatorError;

use ethernet::{Ethernet, NetworkBackend};

use gpio::Gpio;

use i2c::{Eeprom, I2c};

use interrupt_controller::InterruptController;

use plic::Plic;

use revoker::Revoker;

use spi::{LoopbackPeripheral, Spi};

use storage::Storage;

use timer32::Timer32;

use timer64::Timer64;

use uart::{Uart, UartLayout};

macro_rules! registers {
    ($name:ident { $($register:ident = $offset:literal, $sdk:literal;)* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum $name {
            $($register,)*
        }

        impl $name {
            pub(crate) const ALL: &[(Self, &'static str)] = &[$((Self::$register, $sdk),)*];

            pub(crate) const fn offset(self) -> u32 {
                match self {
                    $(Self::$register => $offset,)*
                }
            }

            pub(crate) fn find(offset: u32) -> Option<Self> {
                Self::ALL
                    .iter()
                    .map(|&(register, _)| register)
                    .find(|register| register.offset() == offset)
            }

            pub(crate) fn access(
                offset: u32,
                width: usize,
            ) -> Result<Self, crate::machine::error::EmulatorError> {
                crate::machine::devices::register_width(offset, width, u32::MAX)?;
                Self::find(offset).ok_or_else(|| crate::machine::error::EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "no register at this offset".into(),
                })
            }
        }
    };
}

mod device_activity;

pub(crate) mod ethernet;

pub(crate) mod gpio;

pub(crate) mod i2c;

pub(crate) mod interrupt_controller;

mod interrupt_line;

pub(crate) mod output_capture;

pub(crate) mod plic;

pub(crate) mod revoker;

pub(crate) mod spi;

pub(crate) mod storage;

pub(crate) mod timer32;

pub(crate) mod timer64;

pub(crate) mod uart;

pub(crate) use device_activity::DeviceActivity;

pub(crate) use interrupt_line::InterruptLine;

pub(crate) const DEVICE_KINDS: &[DeviceKind] = &[
    DeviceKind {
        name: "uart",
        size: REGISTERS,
        keys: &[DeviceKey::Layout],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |device| {
            Ok(Box::new(Uart::with_layout(
                device.layout.unwrap_or_default(),
            )))
        },
    },
    DeviceKind {
        name: "16550",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |_| Ok(Box::new(Uart::with_layout(UartLayout::Ns16550))),
    },
    DeviceKind {
        name: "timer64",
        size: 0xc0000,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: false,
        make: |_| Ok(Box::new(Timer64::new())),
    },
    DeviceKind {
        name: "timer32",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |_| Ok(Box::new(Timer32::new())),
    },
    DeviceKind {
        name: "interrupt-controller",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: true,
        routes_interrupts: false,
        make: |_| Ok(Box::new(InterruptController::new())),
    },
    DeviceKind {
        name: "plic",
        size: plic::BYTES,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: true,
        routes_interrupts: false,
        make: |_| Ok(Box::new(Plic::new())),
    },
    DeviceKind {
        name: "gpio",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |_| Ok(Box::new(Gpio::new())),
    },
    DeviceKind {
        name: "spi",
        size: REGISTERS,
        keys: &[DeviceKey::AttachedDevice],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: make_spi,
    },
    DeviceKind {
        name: "i2c",
        size: REGISTERS,
        keys: &[DeviceKey::AttachedDevice],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: make_i2c,
    },
    DeviceKind {
        name: "ethernet",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |_| Ok(Box::new(Ethernet::new(NetworkBackend::InMemory))),
    },
    DeviceKind {
        name: "revoker",
        size: REGISTERS,
        keys: &[],
        needs_cheriot: true,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |_| {
            Ok(Box::new(Revoker::new(
                CheriotProcessor::REVOCABLE_MEMORY_BASE,
                CheriotProcessor::REVOCATION_BITMAP_BASE,
            )))
        },
    },
    DeviceKind {
        name: "storage",
        size: REGISTERS,
        keys: &[DeviceKey::Storage],
        needs_cheriot: false,
        controls_interrupts: false,
        routes_interrupts: true,
        make: |device| {
            Ok(Box::new(Storage::new(device.sectors, device.read_only)?))
        },
    },
];

const REGISTERS: u32 = 0x100;

fn make_i2c(
    device: &DeviceConfiguration,
) -> Result<Box<dyn Device>, EmulatorError> {
    let mut i2c = I2c::new();
    match device.attached_device.as_str() {
        "none" => {}
        "eeprom" => i2c.attach(0x50, Box::new(Eeprom::new()))?,
        _ => {
            return Err(EmulatorError::Configuration(
                "I2C attached_device must be none or eeprom".into(),
            ));
        }
    }
    Ok(Box::new(i2c))
}

fn make_spi(
    device: &DeviceConfiguration,
) -> Result<Box<dyn Device>, EmulatorError> {
    let mut spi = Spi::new();
    match device.attached_device.as_str() {
        "none" => {}
        "loopback" => spi.attach(0, Box::new(LoopbackPeripheral))?,
        _ => {
            return Err(EmulatorError::Configuration(
                "SPI attached_device must be none or loopback".into(),
            ));
        }
    }
    Ok(Box::new(spi))
}

pub(crate) fn register_width(
    offset: u32,
    width: usize,
    bytes: u32,
) -> Result<(), EmulatorError> {
    if width != 4 || !offset.is_multiple_of(4) || offset >= bytes {
        return Err(EmulatorError::Bus {
            address: u64::from(offset),
            explanation: "device requires an aligned 32-bit register access"
                .into(),
        });
    }
    Ok(())
}
