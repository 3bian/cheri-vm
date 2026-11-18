// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::plic::Plic;
use crate::machine::devices::uart::Uart;
use crate::machine::devices::{Device, InterruptLine};

const MACHINE_CLAIM: u32 = 0x20_0004;
const MACHINE_ENABLE: u32 = 0x2000;
const MACHINE_THRESHOLD: u32 = 0x20_0000;
const PENDING: u32 = 0x1000;
const SUPERVISOR_CLAIM: u32 = 0x20_1004;
const SUPERVISOR_ENABLE: u32 = 0x2080;

#[test]
fn a_claim_takes_the_highest_priority_and_holds_it_until_it_is_completed() {
    let mut plic = Plic::new();
    write(&mut plic, 4, 1);
    write(&mut plic, 8, 3);
    write(&mut plic, 12, 3);
    write(&mut plic, SUPERVISOR_ENABLE, 0b1110);
    plic.set_levels(0b1110);

    assert_eq!(read(&mut plic, SUPERVISOR_CLAIM), 2);
    assert_eq!(read(&mut plic, SUPERVISOR_CLAIM), 3);
    assert_eq!(read(&mut plic, SUPERVISOR_CLAIM), 1);
    assert_eq!(read(&mut plic, SUPERVISOR_CLAIM), 0);
    assert!(!plic.interrupt_pending(InterruptLine::SupervisorExternal));

    write(&mut plic, SUPERVISOR_CLAIM, 2);
    assert_eq!(read(&mut plic, PENDING), 0b0100);
    assert_eq!(read(&mut plic, SUPERVISOR_CLAIM), 2);
    assert_eq!(read(&mut plic, MACHINE_CLAIM), 0);
}

#[test]
fn a_priority_is_at_most_seven_source_zero_does_not_exist_and_a_byte_access_is_refused()
 {
    let mut plic = Plic::new();
    write(&mut plic, 4, 0xff);
    assert_eq!(read(&mut plic, 4), 7);
    write(&mut plic, 0, 7);
    assert_eq!(read(&mut plic, 0), 0);
    write(&mut plic, MACHINE_ENABLE, u32::MAX);
    assert_eq!(read(&mut plic, MACHINE_ENABLE), 0x7fff_fffe);
    plic.set_levels(u32::MAX);
    assert_eq!(read(&mut plic, PENDING), 0x7fff_fffe);
    assert!(plic.read(4, 1).is_err());
    assert!(plic.write(0x400_0000, 4, 0).is_err());
}

#[test]
fn a_source_reaches_the_context_that_enables_it_above_its_threshold() {
    let mut plic = Plic::new();
    write(&mut plic, 4, 1);
    write(&mut plic, 8, 3);
    write(&mut plic, SUPERVISOR_ENABLE, 0b110);
    plic.set_levels(0b110);
    assert_eq!(read(&mut plic, PENDING), 0b110);
    assert!(!plic.interrupt_pending(InterruptLine::MachineExternal));
    assert!(plic.interrupt_pending(InterruptLine::SupervisorExternal));

    write(&mut plic, MACHINE_ENABLE, 0b010);
    assert!(plic.interrupt_pending(InterruptLine::MachineExternal));
    write(&mut plic, MACHINE_THRESHOLD, 1);
    assert!(!plic.interrupt_pending(InterruptLine::MachineExternal));
    assert_eq!(read(&mut plic, MACHINE_THRESHOLD), 1);
}

fn read(plic: &mut Plic, offset: u32) -> u32 {
    plic.read(offset, 4).unwrap()
}

#[test]
fn the_uart_of_the_cva6_profile_reaches_the_supervisor_through_source_one() {
    let mut hart = crate::profile::reader::read(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cva6.toml"
    )))
    .unwrap()
    .build_supervisor_raw(0x8000_0000, &[0x13, 0, 0, 0])
    .unwrap();
    let plic = 0x0c00_0000_u64;
    hart.bus.write(plic + 4, 4, 1).unwrap();
    hart.bus
        .write(plic + u64::from(SUPERVISOR_ENABLE), 4, 2)
        .unwrap();
    hart.bus.write(0x1000_0004, 4, 1).unwrap();
    hart.bus.refresh_interrupts();
    assert_eq!(hart.bus.machine_interrupt_bits(), 1 << 7);

    hart.bus
        .device_mut::<Uart>("uart")
        .unwrap()
        .inject_receive(b"x")
        .unwrap();
    hart.bus.refresh_interrupts();
    assert_eq!(hart.bus.machine_interrupt_bits(), 1 << 7 | 1 << 9);
    assert_eq!(
        hart.bus
            .read(plic + u64::from(SUPERVISOR_CLAIM), 4)
            .unwrap(),
        1
    );
    hart.bus.refresh_interrupts();
    assert_eq!(hart.bus.machine_interrupt_bits(), 1 << 7);
}

fn write(plic: &mut Plic, offset: u32, value: u32) {
    plic.write(offset, 4, value).unwrap();
}
