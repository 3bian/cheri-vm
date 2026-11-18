// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod levels;
mod modes;

use crate::hart::execution::TrapPolicy;
use crate::hart::{Hart, Rv64Rvy};
use crate::isa::pointer::rvy::{Capability128, permissions};
use crate::machine::error::{EmulatorError, ExceptionCause};

const BASE: u64 = 0x4000_0000;
const DATA: u64 = BASE + 0x1000;
pub(super) const EBREAK: u32 = 0x0010_0073;
const ECALL: u32 = 0x0000_0073;
pub(super) const MRET: u32 = 0x3020_0073;

const PROFILE: &str = "name = \"capability\"\n[emulator]\nprocessor = \
                       \"rv64\"\nextensions = [\"y\", \"i\", \"m\", \"a\", \
                       \"c\", \"zicsr\", \"zifencei\", \"zba\"]\ntiming = \
                       \"per-step\"\nmemory = { ram = { start = 0x4000_0000, \
                       length = 0x1_0000 } }\n";

#[test]
fn a_capability_keeps_its_tag_through_memory_until_a_byte_of_it_is_written() {
    let mut program = data_capability(2, 32).to_vec();
    program.extend(data_capability(3, 64));
    program.extend([
        store_capability(2, 3, 0),
        load_capability(8, 3, 0),
        ld(13, 3, 8),
        stored(0, 0, 3, 0, 0x23),
        load_capability(9, 3, 0),
        addi(11, 0, 32),
        register(0x13, 11, 3, 10),
        store_capability(2, 10, 16),
        load_capability(12, 3, 16),
        addi(11, 0, 2),
        register(0x13, 11, 3, 14),
        store_capability(2, 3, 32),
        load_capability(15, 14, 32),
        load_capability(16, 3, 8),
    ]);
    let (hart, cause, value) = stopped(&program);
    assert_eq!(cause, ExceptionCause::LoadAccessFault);
    assert_eq!(value, DATA + 8);
    let stored = hart.processor.capability(2);
    assert!(stored.tag);
    assert_eq!(hart.processor.capability(8), stored);
    assert_eq!(hart.processor.read_register_unchecked(13), stored.metadata);
    let overwritten = hart.processor.capability(9);
    assert!(!overwritten.tag);
    assert_eq!(overwritten.metadata, stored.metadata);
    assert_eq!(overwritten.address, DATA & !0xff);
    assert_eq!(
        hart.processor.capability(12),
        Capability128 {
            tag: false,
            ..stored
        }
    );
    let weakened = hart.processor.capability(15);
    assert!(weakened.tag);
    assert_eq!(
        weakened.permissions(),
        stored.permissions()
            & !(permissions::WRITE | permissions::LOAD_MUTABLE)
    );
}

#[test]
fn a_link_is_a_sealed_entry_that_a_return_unseals_and_nothing_else_may_change()
{
    let hart = finished(&[
        0x0000_0297,
        jal(6, 8),
        EBREAK,
        query(5, 6, 7),
        query(4, 6, 8),
        jalr(0, 6, 0),
    ]);
    let derived = hart.processor.capability(5);
    assert!(derived.tag && derived.has_infinite_bounds());
    assert_eq!(derived.address, BASE);
    let link = hart.processor.capability(6);
    assert!(link.tag && link.sealed());
    assert_eq!(link.address, BASE + 8);
    assert_eq!(integers(&hart, 7..=8), [1, 1]);
    assert_eq!(
        hart.processor.machine_registers.exception_program_counter,
        BASE + 8
    );

    for change in [immediate(0, 6, 4, 9, 0x7b), addi(9, 6, 0)] {
        let (hart, cause, value) =
            stopped(&[jal(6, 8), EBREAK, change, jalr(0, 9, 0)]);
        assert_eq!(cause, ExceptionCause::CheriInstructionAccessFault);
        assert_eq!(value, BASE + 4);
        assert!(!hart.processor.capability(9).tag);
    }

    let (_, cause, _) = stopped(&[jal(6, 8), EBREAK, jalr(0, 6, 4), EBREAK]);
    assert_eq!(cause, ExceptionCause::CheriInstructionAccessFault);
}

#[test]
fn a_load_or_store_needs_a_capability_that_covers_it() {
    let mut program = data_capability(2, 32).to_vec();
    program.extend([addi(6, 0, 0x55), sd(6, 2, 8), ld(7, 2, 8), sd(6, 2, 32)]);
    let (hart, cause, value) = stopped(&program);
    assert_eq!(cause, ExceptionCause::CheriStoreAccessFault);
    assert_eq!(value, DATA + 32);
    assert_eq!(hart.processor.read_register_unchecked(7), 0x55);
    assert_eq!(
        hart.processor.capability(2).bounds(),
        (DATA, (DATA + 32).into())
    );

    let (_, cause, value) = stopped(&[data_address(5), ld(7, 5, 0)]);
    assert_eq!(cause, ExceptionCause::CheriLoadAccessFault);
    assert_eq!(value, DATA);

    let mut read_only = data_capability(2, 32).to_vec();
    read_only.extend([
        addi(6, 0, 1),
        register(0x13, 6, 2, 2),
        ld(7, 2, 0),
        sd(6, 2, 0),
    ]);
    let (_, cause, _) = stopped(&read_only);
    assert_eq!(cause, ExceptionCause::CheriStoreAccessFault);

    let (_, cause, value) = stopped(&[ld(7, 0, 0x100)]);
    assert_eq!(cause, ExceptionCause::CheriLoadAccessFault);
    assert_eq!(value, 0x100);
}

#[test]
fn a_short_move_copies_a_capability_and_the_shifted_add_and_atomics_work_on_one()
 {
    let mut program = data_capability(3, 64).to_vec();
    program.extend(data_capability(2, 32));
    program.extend([
        0x0000_848a,
        addi(6, 0, 4),
        register(0x0d, 3, 6, 10),
        store_capability(2, 3, 16),
        immediate(16, 3, 4, 11, 0x7b),
        2 << 27 | 11 << 15 | 3 << 12 | 12 << 7 | 0x7b,
        3 << 27 | 3 << 20 | 11 << 15 | 3 << 12 | 13 << 7 | 0x7b,
        3 << 27 | 3 << 20 | 11 << 15 | 3 << 12 | 14 << 7 | 0x7b,
        1 << 27 | 2 << 20 | 11 << 15 | 3 << 12 | 15 << 7 | 0x7b,
        load_capability(16, 3, 16),
        EBREAK,
    ]);
    program[8] = 0x0000_0001 | 0x848a << 16;
    program.insert(9, 0x0001_0001);
    let hart = finished(&program);
    let narrow = hart.processor.capability(2);
    let wide = hart.processor.capability(3);
    assert_eq!(hart.processor.capability(9), narrow);
    assert_eq!(hart.processor.capability(10), wide.with_address(DATA + 16));
    assert_eq!(hart.processor.capability(12), narrow);
    assert_eq!(integers(&hart, 13..=14), [0, 1]);
    assert_eq!(hart.processor.capability(15), wide);
    assert_eq!(hart.processor.capability(16), narrow);
}

#[test]
fn a_trap_swaps_the_program_counter_capability_with_mtvec_and_mret_gives_it_back()
 {
    let mut hart = machine(&[
        0x0000_0297,
        immediate(32, 5, 4, 5, 0x7b),
        set_bounds(5, 5, 16),
        csrrw(0, 0x305, 5),
        ECALL,
        addi(20, 0, 7),
        EBREAK,
        0,
        csrr(10, 0x341),
        immediate(4, 10, 4, 10, 0x7b),
        csrrw(0, 0x341, 10),
        MRET,
    ]);
    for _ in 0..5 {
        hart.step().unwrap();
    }
    let handler = hart.processor.program_counter_capability();
    assert_eq!(hart.processor.program_counter, BASE + 32);
    assert!(handler.tag);
    assert_eq!(handler.bounds(), (BASE + 32, (BASE + 48).into()));
    assert_eq!(hart.processor.machine_registers.trap_cause, 11);
    for _ in 0..4 {
        hart.step().unwrap();
    }
    let saved = hart.processor.capability(10);
    assert!(saved.tag && saved.has_infinite_bounds());
    assert_eq!(saved.address, BASE + 20);
    assert_eq!(hart.processor.program_counter, BASE + 20);
    assert!(
        hart.processor
            .program_counter_capability()
            .has_infinite_bounds()
    );
    hart.step().unwrap();
    assert_eq!(hart.processor.read_register_unchecked(20), 7);
}

pub(super) fn addi(destination: u32, first: u32, value: i32) -> u32 {
    immediate(value, first, 0, destination, 0x13)
}

#[test]
fn an_integer_written_to_mtvec_leaves_no_capability_to_trap_through() {
    let mut hart = machine(&[
        0x0000_0297,
        addi(5, 5, 16),
        csrrw(0, 0x305, 5),
        ECALL,
        EBREAK,
    ]);
    for _ in 0..4 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.program_counter, BASE + 16);
    assert!(!hart.processor.program_counter_capability().tag);
    hart.step().unwrap();
    assert_eq!(hart.processor.machine_registers.trap_cause, 32);

    let moved =
        finished(&[immediate(0x305, 8, 5, 0, 0x73), csrr(10, 0x305), EBREAK]);
    let vector = moved.processor.capability(10);
    assert!(vector.tag);
    assert_eq!(vector.address, 8);
}

#[test]
fn code_runs_only_inside_the_bounds_of_the_capability_it_was_entered_through() {
    let (hart, cause, value) = stopped(&[
        0x0000_0297,
        immediate(16, 5, 4, 5, 0x7b),
        set_bounds(5, 5, 8),
        jalr(1, 5, 0),
        addi(10, 0, 1),
        addi(11, 0, 2),
        addi(12, 0, 3),
    ]);
    assert_eq!(cause, ExceptionCause::CheriInstructionAccessFault);
    assert_eq!(value, BASE + 24);
    assert_eq!(integers(&hart, 10..=12), [1, 2, 0]);
    let link = hart.processor.capability(1);
    assert!(link.sealed() && link.has_infinite_bounds());
}

pub(super) fn csrr(destination: u32, address: i32) -> u32 {
    immediate(address, 0, 2, destination, 0x73)
}

pub(super) fn csrrw(destination: u32, address: i32, source: u32) -> u32 {
    immediate(address, source, 1, destination, 0x73)
}

fn data_address(destination: u32) -> u32 {
    0x4000_1000 | destination << 7 | 0x37
}

fn data_capability(destination: u32, length: i32) -> [u32; 4] {
    [
        csrr(1, 0x305),
        data_address(5),
        register(0x0b, 5, 1, destination),
        set_bounds(destination, destination, length),
    ]
}

fn finished(words: &[u32]) -> Hart<Rv64Rvy> {
    let (hart, cause, _) = stopped(words);
    assert_eq!(cause, ExceptionCause::Breakpoint);
    hart
}

pub(super) fn immediate(
    value: i32,
    first: u32,
    function_code: u32,
    destination: u32,
    opcode: u32,
) -> u32 {
    (value as u32 & 0xfff) << 20
        | first << 15
        | function_code << 12
        | destination << 7
        | opcode
}

fn integers(
    hart: &Hart<Rv64Rvy>,
    range: std::ops::RangeInclusive<u8>,
) -> Vec<u64> {
    range
        .map(|index| hart.processor.read_register_unchecked(index))
        .collect()
}

fn jal(destination: u32, offset: u32) -> u32 {
    (offset >> 1 & 0x3ff) << 21
        | (offset >> 11 & 1) << 20
        | destination << 7
        | 0x6f
}

fn jalr(destination: u32, first: u32, value: i32) -> u32 {
    immediate(value, first, 0, destination, 0x67)
}

fn ld(destination: u32, first: u32, value: i32) -> u32 {
    immediate(value, first, 3, destination, 0x03)
}

pub(super) fn load_capability(destination: u32, first: u32, value: i32) -> u32 {
    immediate(value, first, 1, destination, 0x7b)
}

fn machine(words: &[u32]) -> Hart<Rv64Rvy> {
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    crate::profile::reader::read(PROFILE)
        .unwrap()
        .build_capability_raw(BASE, &firmware)
        .unwrap()
}

fn query(code: u32, first: u32, destination: u32) -> u32 {
    register(0x7a, code, first, destination)
}

pub(super) fn register(
    prefix: u32,
    second: u32,
    first: u32,
    destination: u32,
) -> u32 {
    prefix << 25 | second << 20 | first << 15 | destination << 7 | 0x7b
}

fn sd(second: u32, first: u32, value: i32) -> u32 {
    stored(value, second, first, 3, 0x23)
}

fn set_bounds(destination: u32, first: u32, length: i32) -> u32 {
    immediate(0xe00 | length, first, 5, destination, 0x7b)
}

fn stopped(words: &[u32]) -> (Hart<Rv64Rvy>, ExceptionCause, u64) {
    let mut hart = machine(words);
    hart.trap_policy = TrapPolicy::Stop;
    for _ in 0..200 {
        match hart.step() {
            Ok(_) => (),
            Err(EmulatorError::Trap { cause, value, .. }) => {
                return (hart, cause, value);
            }
            Err(error) => panic!("{error}"),
        }
    }
    panic!("no trap was reached");
}

pub(super) fn store_capability(second: u32, first: u32, value: i32) -> u32 {
    stored(value, second, first, 2, 0x7b)
}

pub(super) fn stored(
    value: i32,
    second: u32,
    first: u32,
    function_code: u32,
    opcode: u32,
) -> u32 {
    let value = value as u32 & 0xfff;
    (value >> 5) << 25
        | second << 20
        | first << 15
        | function_code << 12
        | (value & 31) << 7
        | opcode
}

#[test]
fn system_registers_and_trap_returns_need_the_system_permission() {
    let entered = |body: &[u32]| {
        let mut program = vec![
            0x0000_0297,
            immediate(28, 5, 4, 5, 0x7b),
            0x0001_0337,
            register(0x13, 6, 5, 5),
            csrr(0, 0x300),
            jalr(1, 5, 0),
            0,
        ];
        program.extend_from_slice(body);
        stopped(&program)
    };
    let (hart, cause, _) =
        entered(&[csrr(10, 0x07f), csrr(11, 0xc02), csrr(12, 0x300)]);
    assert_eq!(cause, ExceptionCause::IllegalInstruction);
    assert_eq!(
        hart.processor.machine_registers.exception_program_counter,
        BASE + 36
    );
    let (_, cause, _) = entered(&[csrrw(0, 0x07f, 1)]);
    assert_eq!(cause, ExceptionCause::IllegalInstruction);
    let (_, cause, _) = entered(&[MRET]);
    assert_eq!(cause, ExceptionCause::IllegalInstruction);

    let hart = finished(&[
        csrr(1, 0x305),
        csrrw(0, 0x37f, 1),
        csrr(10, 0x37f),
        EBREAK,
    ]);
    assert_eq!(hart.processor.capability(10), Capability128::INFINITE);
}

#[test]
fn the_machine_starts_with_one_capability_in_mtvec_and_an_integer_copy_has_no_tag()
 {
    let hart = finished(&[
        csrr(1, 0x305),
        addi(2, 1, 0),
        register(0x03, 0, 1, 3),
        query(4, 1, 4),
        query(4, 2, 5),
        query(1, 1, 6),
        query(0, 1, 7),
        query(2, 1, 8),
        query(3, 1, 9),
        query(7, 1, 10),
        query(5, 1, 11),
        csrr(12, 0x301),
        EBREAK,
    ]);
    assert_eq!(hart.processor.capability(1), Capability128::INFINITE);
    assert_eq!(hart.processor.capability(2), Capability128::default());
    assert_eq!(hart.processor.capability(3), Capability128::INFINITE);
    assert_eq!(
        integers(&hart, 4..=11),
        [
            1,
            0,
            permissions::ALL | permissions::RESERVED_ONES | permissions::LEVELS,
            0,
            u64::MAX,
            u64::MAX,
            Capability128::INFINITE.metadata,
            0,
        ]
    );
    assert_eq!(hart.processor.capability(4), Capability128::integer(1));
    assert_eq!(
        hart.processor.read_register_unchecked(12),
        0x8000_0000_0100_1105
    );
    let counter = hart.processor.program_counter_capability();
    assert!(counter.tag && counter.has_infinite_bounds());
}
