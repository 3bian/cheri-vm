// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::bus::*;
use crate::machine::devices::InterruptLine;
use crate::machine::devices::uart::{Uart, UartLayout};
use crate::machine::memory::MemoryRegion;
use crate::machine::memory::{MemoryKind, Tagging};

impl Bus {
    pub(crate) fn drain_output(&mut self, output: &mut Vec<u8>) {
        self.touch();
        for mapping in &mut self.devices {
            mapping.device.drain_output(output);
        }
    }

    pub(crate) fn remove_memory(&mut self, name: &str) -> Option<MemoryRegion> {
        let index =
            self.memory.iter().position(|region| region.name == name)?;
        Some(self.memory.remove(index))
    }

    pub(crate) fn set_memory_executable(
        &mut self,
        name: &str,
        executable: bool,
    ) -> Result<(), EmulatorError> {
        let region = self
            .memory
            .iter_mut()
            .find(|region| region.name == name)
            .ok_or_else(|| {
                EmulatorError::Configuration(format!("no memory named {name}"))
            })?;
        region.executable = executable;
        Ok(())
    }
}

#[test]
fn crossing_and_overlapping_regions_are_rejected() {
    let mut bus = Bus::new();
    bus.add_memory(
        MemoryRegion::new(
            "one".into(),
            0x1000,
            64,
            MemoryKind::Ram,
            Tagging::None,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(
        bus.add_memory(
            MemoryRegion::new(
                "two".into(),
                0x103f,
                64,
                MemoryKind::Ram,
                Tagging::None
            )
            .unwrap()
        )
        .is_err()
    );
    assert!(bus.read(0x103f, 4).is_err());
}

#[test]
fn output_draining_preserves_device_order_input_and_capture_limits() {
    let mut bus = Bus::new();
    bus.add_device(
        "console".into(),
        0x1000,
        256,
        Box::new(Uart::with_layout(UartLayout::Ns16550)),
    )
    .unwrap();
    let mut uart = Uart::new();
    uart.inject_receive(b"input").unwrap();
    bus.add_device("uart".into(), 0x2000, 12, Box::new(uart))
        .unwrap();
    bus.write(0x2008, 4, 1).unwrap();

    for _ in 0..65536 {
        bus.write(0x1000, 4, 0xff).unwrap();
        bus.write(0x2000, 4, b'U'.into()).unwrap();
    }

    assert!(bus.write(0x1000, 4, 0).is_err());
    assert!(bus.write(0x2000, 4, 0).is_err());
    assert_eq!(bus.read(0x2004, 4).unwrap(), 1);
    assert!(bus.interrupt_pending(InterruptLine::MachineExternal));
    bus.check_host_errors().unwrap();

    let mut output = vec![0];
    bus.drain_output(&mut output);
    assert_eq!(output.len(), 131073);
    assert_eq!(output[0], 0);
    assert!(output[1..65537].iter().all(|byte| *byte == 0xff));
    assert!(output[65537..].iter().all(|byte| *byte == b'U'));
    assert_eq!(bus.read(0x2004, 4).unwrap(), 3);
    assert!(bus.interrupt_pending(InterruptLine::MachineExternal));
    assert_eq!(bus.read(0x2000, 4).unwrap(), u32::from(b'i'));

    bus.drain_output(&mut output);
    assert_eq!(output.len(), 131073);
    bus.write(0x1000, 4, b'C'.into()).unwrap();
    bus.write(0x2000, 4, b'U'.into()).unwrap();
    output.clear();
    bus.drain_output(&mut output);
    assert_eq!(output, b"CU");
}
