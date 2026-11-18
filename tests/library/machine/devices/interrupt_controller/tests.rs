// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::InterruptLine;
use crate::machine::devices::interrupt_controller::registers::Register;
use crate::machine::devices::uart::Uart;
use crate::machine::devices::uart::registers::Register as UartRegister;

const CONTROLLER: u32 = 0x0c00_0000;
const UART: u32 = 0x1000_0000;

#[test]
fn a_routed_interrupt_is_claimed_completed_and_checked_by_source() {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/rv32imc.toml"
    ));
    let mut c = crate::profile::reader::read(text)
        .unwrap()
        .build(0x8000_0000)
        .unwrap();
    c.bus
        .device_mut::<Uart>("uart")
        .unwrap()
        .inject_receive(b"x")
        .unwrap();
    c.bus
        .write((UART + UartRegister::Control.offset()).into(), 4, 1)
        .unwrap();
    c.bus.refresh_interrupts();
    assert!(!c.bus.interrupt_pending(InterruptLine::MachineExternal));
    c.bus
        .write((CONTROLLER + Register::Enabled.offset()).into(), 4, 1 << 1)
        .unwrap();
    assert!(c.bus.interrupt_pending(InterruptLine::MachineExternal));
    assert_eq!(
        c.bus
            .read((CONTROLLER + Register::ClaimComplete.offset()).into(), 4)
            .unwrap(),
        1
    );
    assert!(!c.bus.interrupt_pending(InterruptLine::MachineExternal));
    assert!(
        c.bus
            .write((CONTROLLER + Register::ClaimComplete.offset()).into(), 4, 2)
            .is_err()
    );
    c.bus
        .write((CONTROLLER + Register::ClaimComplete.offset()).into(), 4, 1)
        .unwrap();
    assert!(c.bus.interrupt_pending(InterruptLine::MachineExternal));
    c.bus
        .read((UART + UartRegister::Data.offset()).into(), 4)
        .unwrap();
    c.bus.refresh_interrupts();
    assert!(!c.bus.interrupt_pending(InterruptLine::MachineExternal));
    assert!(
        crate::profile::reader::read(
            &text.replace("interrupt_source = 1", "interrupt_source = 32")
        )
        .unwrap()
        .build(0x8000_0000)
        .is_err()
    );
}
