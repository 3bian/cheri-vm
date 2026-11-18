// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{
    EBREAK, MRET, addi, csrr, csrrw, immediate, load_capability, register,
    store_capability,
};
use crate::hart::execution::TrapPolicy;
use crate::hart::privilege::{Mode, PrivilegeModel};
use crate::hart::{Cva6Cheri, Hart};
use crate::isa::extension::rvy::draft;
use crate::isa::pointer::rvy::Capability128;
use crate::machine::TimingPolicy;
use crate::machine::error::{EmulatorError, ExceptionCause};

const BASE: u64 = 0x8000_0000;
const DIRTY: u64 = 8;
const ENABLE: u64 = 1 << 59;
const GENERATION: u64 = 2;
const MAPPED: u64 = 0x4000_3000;

const PROFILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/profiles/cva6-cheri.toml"
));

const READABLE: u64 = 1;
const SUPERVISOR_GENERATION: u64 = 1 << 60;
const USER_GENERATION: u64 = 1 << 61;
const WRITABLE: u64 = 4;

#[test]
fn a_delegated_capability_fault_is_taken_in_supervisor_mode_through_stvec() {
    let mut hart = machine(&[
        0x0000_0297,
        immediate(80, 5, 4, 6, 0x7b),
        csrrw(0, 0x105, 6),
        csrrw(0, 0x302, 7),
        csrrw(0, 0x3b0, 8),
        csrrw(0, 0x3a0, 9),
        immediate(64, 5, 4, 10, 0x7b),
        csrrw(0, 0x341, 10),
        immediate(0x300, 11, 2, 0, 0x73),
        MRET,
        0,
        0,
        0,
        0,
        0,
        0,
        immediate(0, 0, 3, 12, 0x03),
        0,
        0,
        0,
        csrr(20, 0x142),
        csrr(21, 0x141),
        csrr(22, 0x143),
        immediate(0, 21, 4, 21, 0x7b),
        csrrw(0, 0x141, 21),
        0x1020_0073,
    ]);
    for (register, value) in
        [(7, 1 << 33), (8, u64::MAX), (9, 0x1f), (11, 1 << 11)]
    {
        hart.processor.write_register_unchecked(register, value);
    }
    for _ in 0..10 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.privilege.mode(), Mode::Supervisor);
    assert_eq!(hart.processor.program_counter, BASE + 64);
    for _ in 0..4 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.privilege.mode(), Mode::Supervisor);
    assert_eq!(hart.processor.read_register_unchecked(20), 33);
    assert_eq!(hart.processor.read_register_unchecked(22), 0);
    let saved = hart.processor.capability(21);
    assert!(saved.tag && saved.has_infinite_bounds());
    assert_eq!(saved.address, BASE + 64);
    for _ in 0..3 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.program_counter, BASE + 64);
    assert!(hart.processor.program_counter_capability().tag);
    assert_eq!(hart.processor.privilege.mode(), Mode::Supervisor);
}

#[test]
fn a_page_without_the_capability_bit_gives_no_tag_and_takes_none() {
    let mut hart = machine(&[
        csrr(1, 0x305),
        register(0x0b, 5, 1, 2),
        register(0x0b, 9, 1, 3),
        csrrw(0, 0x3b0, 7),
        csrrw(0, 0x3a0, 8),
        csrrw(0, 0x180, 4),
        immediate(0x300, 6, 2, 0, 0x73),
        store_capability(2, 2, 0),
        load_capability(10, 2, 0),
        load_capability(11, 3, 0),
        store_capability(0, 3, 16),
        addi(12, 0, 1),
        store_capability(2, 3, 16),
    ]);
    let table = BASE + 0x2000;
    let leaf = (BASE >> 12) << 10 | 0xc7;
    for (index, entry) in [(1, leaf), (2, leaf | 1 << 58)] {
        hart.bus.write(table + index * 8, 4, entry as u32).unwrap();
        hart.bus
            .write(table + index * 8 + 4, 4, (entry >> 32) as u32)
            .unwrap();
    }
    for (register, value) in [
        (4, 8 << 60 | table >> 12),
        (5, BASE + 0x3000),
        (6, 1 << 17 | 1 << 11),
        (7, u64::MAX),
        (8, 0x1f),
        (9, 0x4000_3000),
    ] {
        hart.processor.write_register_unchecked(register, value);
    }
    hart.trap_policy = TrapPolicy::Stop;
    let fault = (0..20).find_map(|_| hart.step().err());
    assert!(
        matches!(
            fault,
            Some(EmulatorError::Trap {
                cause: ExceptionCause::CheriStorePageFault,
                value: 0x4000_3010,
                ..
            })
        ),
        "{fault:?}"
    );
    assert_eq!(hart.processor.read_register_unchecked(12), 1);
    let stored = hart.processor.capability(2);
    assert!(stored.tag);
    assert_eq!(hart.processor.capability(10), stored);
    assert_eq!(
        hart.processor.capability(11),
        Capability128 {
            tag: false,
            ..stored
        }
    );
}

#[test]
fn a_read_generation_decides_whether_a_tagged_load_traps() {
    let load = load_capability(10, 2, 0);
    let untagged_load = load_capability(10, 2, 16);
    for (field, status, expected) in [
        (0, ENABLE, Ok(false)),
        (GENERATION, ENABLE, Ok(true)),
        (READABLE, ENABLE, Ok(true)),
        (
            READABLE | GENERATION,
            ENABLE | SUPERVISOR_GENERATION,
            Ok(true),
        ),
        (READABLE | GENERATION, ENABLE, Err(())),
        (READABLE, ENABLE | SUPERVISOR_GENERATION, Err(())),
        (READABLE | GENERATION, ENABLE | USER_GENERATION, Err(())),
        (READABLE | GENERATION | WRITABLE, 0, Ok(false)),
        (DIRTY | READABLE | GENERATION, 0, Ok(true)),
        (DIRTY, ENABLE, Ok(false)),
    ] {
        let (hart, cause, value) = through_a_page(PROFILE, field, status, load);
        let case = format!("field {field:#06b} status {status:#x}");
        match expected {
            Ok(tag) => {
                assert_eq!(cause, ExceptionCause::Breakpoint, "{case}");
                assert_eq!(hart.processor.capability(10).tag, tag, "{case}");
                assert_eq!(hart.processor.capability(10).address, BASE + 0x100);
                assert_eq!(
                    hart.processor.read_register_unchecked(20) >> 59 & 7,
                    status >> 59,
                    "{case}"
                );
            }
            Err(()) => {
                assert_eq!(
                    cause,
                    ExceptionCause::CheriLoadCapabilityFault,
                    "{case}"
                );
                assert_eq!(cause.code(), 35);
                assert_eq!(value, MAPPED, "{case}");
                assert!(!hart.processor.capability(10).tag);
            }
        }
        let (hart, cause, _) =
            through_a_page(PROFILE, field, status, untagged_load);
        assert_eq!(cause, ExceptionCause::Breakpoint, "{case}");
        assert!(!hart.processor.capability(10).tag);
    }
}

#[test]
fn a_tagged_store_needs_a_writable_page_already_marked_dirty() {
    let store = store_capability(3, 2, 32);
    let untagged_store = store_capability(0, 2, 32);
    for (field, status, stored) in [
        (WRITABLE | DIRTY, ENABLE, true),
        (WRITABLE | DIRTY | READABLE, ENABLE, true),
        (WRITABLE, ENABLE, false),
        (DIRTY, ENABLE, false),
        (0, ENABLE, false),
        (DIRTY, 0, true),
        (WRITABLE, 0, false),
    ] {
        let (hart, cause, value) =
            through_a_page(PROFILE, field, status, store);
        let case = format!("field {field:#06b} status {status:#x}");
        if stored {
            assert_eq!(cause, ExceptionCause::Breakpoint, "{case}");
            assert!(hart.bus.tag_at(BASE + 0x3020), "{case}");
        } else {
            assert_eq!(cause, ExceptionCause::CheriStorePageFault, "{case}");
            assert_eq!(value, MAPPED + 32, "{case}");
            assert!(!hart.bus.tag_at(BASE + 0x3020), "{case}");
        }
        let (hart, cause, _) =
            through_a_page(PROFILE, field, status, untagged_store);
        assert_eq!(cause, ExceptionCause::Breakpoint, "{case}");
        assert!(!hart.bus.tag_at(BASE + 0x3020));
    }
}

fn machine(words: &[u32]) -> Hart<Cva6Cheri> {
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut configuration = crate::profile::reader::read(PROFILE).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    configuration
        .build_cheri_supervisor_raw(BASE, &firmware)
        .unwrap()
}

#[test]
fn the_profile_names_the_draft_and_the_format_it_is_built_to() {
    assert!(PROFILE.contains(draft::DRAFT));
    assert!(PROFILE.contains(draft::FORMAT));
    assert!(draft::DRAFT_TAG.starts_with(draft::DRAFT));
    let hart = machine(&[EBREAK]);
    assert_eq!(
        hart.processor.machine_registers.isa >> 24 & 1,
        1,
        "misa lacks Y"
    );
    assert!(
        hart.processor
            .program_counter_capability()
            .has_infinite_bounds()
    );
}

fn through_a_page(
    profile: &str,
    field: u64,
    status: u64,
    access: u32,
) -> (Hart<Cva6Cheri>, ExceptionCause, u64) {
    let words = [
        csrrw(0, 0x3b0, 7),
        csrrw(0, 0x3a0, 8),
        csrrw(0, 0x180, 4),
        immediate(0x300, 6, 2, 0, 0x73),
        access,
        csrr(20, 0x100),
        EBREAK,
    ];
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut configuration = crate::profile::reader::read(profile).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    let mut hart = configuration
        .build_cheri_supervisor_raw(BASE, &firmware)
        .unwrap();
    let table = BASE + 0x2000;
    let entry = (BASE >> 12) << 10 | 0xc7 | field << 55;
    hart.bus.write(table + 8, 4, entry as u32).unwrap();
    hart.bus.write(table + 12, 4, (entry >> 32) as u32).unwrap();
    let root = hart.processor.program_counter_capability();
    let kept = root.with_address(BASE + 0x100).with_bounds(32, true);
    let held = BASE + 0x3000;
    for (offset, half) in [(0, kept.address), (8, kept.metadata)] {
        hart.bus.write(held + offset, 4, half as u32).unwrap();
        hart.bus
            .write(held + offset + 4, 4, (half >> 32) as u32)
            .unwrap();
    }
    hart.bus.set_tag(held);
    for (register, value) in [
        (4, 8 << 60 | table >> 12),
        (6, 1 << 17 | 1 << 11 | status),
        (7, u64::MAX),
        (8, 0x1f),
    ] {
        hart.processor.write_register_unchecked(register, value);
    }
    hart.processor.set_capability(2, root.with_address(MAPPED));
    hart.processor.set_capability(3, kept);
    hart.trap_policy = TrapPolicy::Stop;
    for _ in 0..10 {
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

#[test]
fn without_the_extension_the_field_keeps_one_bit_and_the_status_bits_stay_clear()
 {
    let plain = PROFILE.replace("    \"svyrg\",\n", "");
    assert_ne!(plain, PROFILE);
    let load = load_capability(10, 2, 0);
    let all = ENABLE | SUPERVISOR_GENERATION | USER_GENERATION;
    let (hart, cause, _) = through_a_page(&plain, DIRTY, all, load);
    assert_eq!(cause, ExceptionCause::Breakpoint);
    assert!(hart.processor.capability(10).tag);
    assert_eq!(hart.processor.read_register_unchecked(20) >> 59 & 7, 0);
    assert_eq!(hart.processor.machine_registers.machine_status >> 59 & 7, 0);
    for field in [READABLE, GENERATION, WRITABLE, DIRTY | READABLE] {
        let (_, cause, value) = through_a_page(&plain, field, 0, load);
        assert_eq!(cause, ExceptionCause::LoadPageFault, "{field:#06b}");
        assert_eq!(value, MAPPED);
    }

    let delegates = |profile: &str| {
        let mut hart = crate::profile::reader::read(profile)
            .unwrap()
            .build_cheri_supervisor_raw(BASE, &EBREAK.to_le_bytes())
            .unwrap();
        let processor = &mut hart.processor;
        processor
            .privilege
            .write_csr(&mut processor.machine_registers, 0x302, u64::MAX)
            .unwrap();
        processor
            .privilege
            .read_csr::<u64>(&processor.machine_registers, 0x302)
            .unwrap()
            >> 32
    };
    assert_eq!(delegates(PROFILE), 0x1f);
    assert_eq!(delegates(&plain), 0x17);
}
