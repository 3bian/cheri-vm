// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::machine_registers::MachineRegisters;
use crate::hart::privilege::{MachineOnly, Mode, PrivilegeModel, Supervisor};
use crate::isa::extension::ExtensionSet;
use crate::machine::error::ExceptionCause;

const EXCEPTION_DELEGATION: u16 = 0x302;
const INTERRUPT_DELEGATION: u16 = 0x303;
const MACHINE_INTERRUPT_ENABLE: u16 = 0x304;
const MACHINE_INTERRUPT_PENDING: u16 = 0x344;
const MACHINE_STATUS: u16 = 0x300;
const SUPERVISOR_CAUSE: u16 = 0x142;
const SUPERVISOR_INTERRUPT_ENABLE: u16 = 0x104;
const SUPERVISOR_INTERRUPT_PENDING: u16 = 0x144;
const SUPERVISOR_STATUS: u16 = 0x100;
const SUPERVISOR_VECTOR: u16 = 0x105;
const TRANSLATION: u16 = 0x180;

#[test]
fn a_counter_is_hidden_from_a_lower_mode_until_it_is_enabled_and_time_is_not_a_register()
 {
    let (mut model, mut registers) = machine();
    assert_eq!(model.read_csr(&registers, 0xc01), None);
    assert_eq!(model.read_csr(&registers, 0xc00), Some(0_u64));
    assert!(model.write_csr(&mut registers, 0xc00, 1).is_err());
    assert!(model.write_csr(&mut registers, 0xf11, 1).is_err());
    model.write_csr(&mut registers, 0x306, 1).unwrap();
    model
        .write_csr(&mut registers, MACHINE_STATUS, 1 << 11)
        .unwrap();
    model.machine_return(&mut registers);
    assert_eq!(model.read_csr(&registers, 0xc00), Some(0_u64));
    assert_eq!(model.read_csr(&registers, 0xc02), None);
}

#[test]
fn a_delegated_trap_goes_to_the_supervisor_and_returns_with_sret() {
    let (mut model, mut registers) = machine();
    model
        .write_csr(&mut registers, EXCEPTION_DELEGATION, u64::MAX)
        .unwrap();
    assert_eq!(
        model.read_csr(&registers, EXCEPTION_DELEGATION),
        Some(0xb3ff_u64)
    );
    model
        .write_csr(&mut registers, SUPERVISOR_VECTOR, 0x8000_3001)
        .unwrap();
    assert_eq!(
        model.enter_trap(&mut registers, 0x8000_0000, 8, 0, false),
        registers.trap_vector
    );

    let (mut delegating, mut held) = machine();
    delegating
        .write_csr(&mut held, EXCEPTION_DELEGATION, 1 << 8)
        .unwrap();
    delegating
        .write_csr(&mut held, SUPERVISOR_VECTOR, 0x8000_3001)
        .unwrap();
    delegating.write_csr(&mut held, MACHINE_STATUS, 2).unwrap();
    assert_eq!(delegating.machine_return(&mut held), Some(0));
    assert_eq!(
        delegating.enter_trap(&mut held, 0x4000, 8, 0, false),
        0x8000_3000
    );
    assert_eq!(delegating.mode(), Mode::Supervisor);
    assert_eq!(delegating.read_csr(&held, SUPERVISOR_CAUSE), Some(8_u64));
    assert_eq!(held.trap_cause, 0);
    let status: u64 = delegating.read_csr(&held, SUPERVISOR_STATUS).unwrap();
    assert_eq!(status & 0x122, 0x20);
    assert_eq!(delegating.supervisor_return(&mut held), Some(0x4000));
    assert_eq!(delegating.mode(), Mode::User);
}

#[test]
fn a_return_enters_the_mode_the_status_holds_and_a_trap_comes_back() {
    let (mut model, mut registers) = entered(0, 0);
    assert_eq!(model.mode(), Mode::User);
    assert_eq!(
        model.environment_call(),
        ExceptionCause::EnvironmentCallFromUser
    );
    assert_eq!(model.machine_return(&mut registers), None);
    assert_eq!(model.read_csr(&registers, MACHINE_STATUS), None);

    registers.trap_vector = 0x8000_2000;
    assert_eq!(
        model.enter_trap(&mut registers, 0x8000_1004, 8, 0, false),
        0x8000_2000
    );
    assert_eq!(model.mode(), Mode::Machine);
    assert_eq!(registers.trap_cause, 8);
    assert_eq!(registers.machine_status & (3 << 11), 0);
}

#[test]
fn an_interrupt_is_taken_by_the_mode_it_belongs_to_when_that_mode_allows() {
    let (mut model, mut registers) = machine();
    model
        .write_csr(&mut registers, MACHINE_INTERRUPT_ENABLE, 0xaaa)
        .unwrap();
    assert_eq!(model.interrupt(&mut registers, 1 << 7), None);
    model.write_csr(&mut registers, MACHINE_STATUS, 8).unwrap();
    assert_eq!(model.interrupt(&mut registers, 1 << 7), Some(7));
    assert_eq!(model.interrupt(&mut registers, 1 << 7 | 1 << 11), Some(11));

    model
        .write_csr(&mut registers, INTERRUPT_DELEGATION, u64::MAX)
        .unwrap();
    assert_eq!(
        model.read_csr(&registers, INTERRUPT_DELEGATION),
        Some(0x222_u64)
    );
    model
        .write_csr(&mut registers, MACHINE_INTERRUPT_PENDING, 1 << 5)
        .unwrap();
    assert_eq!(model.interrupt(&mut registers, 0), None);
    assert_eq!(
        model.read_csr(&registers, SUPERVISOR_INTERRUPT_PENDING),
        Some(32_u64)
    );

    let (mut model, mut registers) = entered(1, 0);
    model.enter_trap(&mut registers, 0, 2, 0, false);
    model
        .write_csr(&mut registers, INTERRUPT_DELEGATION, 0x222)
        .unwrap();
    model
        .write_csr(&mut registers, MACHINE_INTERRUPT_ENABLE, 0xaaa)
        .unwrap();
    model
        .write_csr(&mut registers, MACHINE_INTERRUPT_PENDING, 1 << 5)
        .unwrap();
    model
        .write_csr(&mut registers, MACHINE_STATUS, 1 << 11)
        .unwrap();
    model.machine_return(&mut registers);
    assert_eq!(model.mode(), Mode::Supervisor);
    assert_eq!(model.interrupt(&mut registers, 0), None);
    assert_eq!(model.interrupt(&mut registers, 1 << 7), Some(7));
    model
        .write_csr(&mut registers, SUPERVISOR_STATUS, 2)
        .unwrap();
    assert_eq!(model.interrupt(&mut registers, 0), Some(5));
    assert_eq!(
        model.read_csr(&registers, SUPERVISOR_INTERRUPT_ENABLE),
        Some(0x222_u64)
    );
    assert_eq!(model.enter_trap(&mut registers, 0x100, 5, 0, true) & 3, 0);
    assert_eq!(
        model.read_csr(&registers, SUPERVISOR_CAUSE),
        Some(1_u64 << 63 | 5)
    );
}

fn entered(previous: u64, status: u64) -> (Supervisor, MachineRegisters<u64>) {
    let (mut model, mut registers) = machine();
    model
        .write_csr(&mut registers, MACHINE_STATUS, previous << 11 | status)
        .unwrap();
    registers.exception_program_counter = 0x8000_1000;
    assert_eq!(model.machine_return(&mut registers), Some(0x8000_1000));
    (model, registers)
}

fn machine() -> (Supervisor, MachineRegisters<u64>) {
    let mut registers = MachineRegisters::default();
    let model = Supervisor::new(ExtensionSet::default(), &mut registers);
    (model, registers)
}

#[test]
fn machine_only_keeps_one_mode_and_refuses_what_the_other_modes_bring() {
    let mut registers = MachineRegisters::<u32>::default();
    let mut model = MachineOnly::new(ExtensionSet::default(), &mut registers);
    assert_eq!(model.mode(), Mode::Machine);
    assert_eq!(registers.machine_status, 3 << 11);
    assert_eq!(model.environment_call(), ExceptionCause::EnvironmentCall);
    assert_eq!(model.supervisor_return(&mut registers), None);
    assert!(!model.fence(&registers));
    assert!(model.waits(&registers));
    assert_eq!(model.read_csr(&registers, SUPERVISOR_STATUS), None);
}

#[test]
fn the_status_bits_that_trap_the_supervisor_do() {
    let (mut model, mut registers) = entered(1, 7 << 20);
    assert_eq!(model.mode(), Mode::Supervisor);
    assert_eq!(model.read_csr(&registers, TRANSLATION), None);
    assert!(model.write_csr(&mut registers, TRANSLATION, 0).is_err());
    assert!(!model.fence(&registers));
    assert!(!model.waits(&registers));
    assert_eq!(model.supervisor_return(&mut registers), None);

    let (mut model, registers) = entered(1, 0);
    assert_eq!(model.read_csr(&registers, TRANSLATION), Some(0_u64));
    assert!(model.fence(&registers));
    assert!(model.waits(&registers));

    let (mut model, registers) = entered(0, 0);
    assert!(!model.fence(&registers));
    assert!(!model.waits(&registers));
}

#[test]
fn the_status_keeps_a_previous_mode_that_exists_and_reports_both_widths() {
    let (mut model, mut registers) = machine();
    model
        .write_csr(&mut registers, MACHINE_STATUS, 1 << 11)
        .unwrap();
    model
        .write_csr(&mut registers, MACHINE_STATUS, 2 << 11)
        .unwrap();
    let status: u64 = model.read_csr(&registers, MACHINE_STATUS).unwrap();
    assert_eq!(status, (1 << 11) | (2 << 32) | (2 << 34));
}
