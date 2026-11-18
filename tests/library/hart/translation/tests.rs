// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::privilege::pmp::Pmp;
use crate::hart::privilege::{Mode, Reach};
use crate::hart::translation::Translation;
use crate::machine::bus::Bus;
use crate::machine::error::ExceptionCause;

const ACCESSED: u64 = 64;
const DIRTY: u64 = 128;
const EXECUTABLE_READABLE: u64 = 1 << 19;
const EXECUTE: u64 = 8;
const LAST: u64 = 0x8001_2000;
const MIDDLE: u64 = 0x8001_1000;
const READ: u64 = 2;
const ROOT: u64 = 0x8001_0000;
const SUPERVISOR_USER_MEMORY: u64 = 1 << 18;
const USER: u64 = 16;
const VALID: u64 = 1;
const WRITE: u64 = 4;

#[test]
fn a_clear_accessed_bit_faults_and_a_clear_dirty_bit_faults_a_store() {
    let pmp = open();
    let (mut bus, mut translation) = mapped(READ | WRITE);
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5000,
            Reach::Load,
            Mode::Supervisor,
            0
        ),
        Err(ExceptionCause::LoadPageFault)
    );
    entry(
        &mut bus,
        LAST,
        5,
        leaf(0x8002_0000, READ | WRITE | ACCESSED),
    );
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5000,
            Reach::Load,
            Mode::Supervisor,
            0
        ),
        Ok(0x8002_0000)
    );
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5000,
            Reach::Store,
            Mode::Supervisor,
            0
        ),
        Err(ExceptionCause::StorePageFault)
    );
    entry(
        &mut bus,
        LAST,
        5,
        leaf(0x8002_0000, READ | WRITE | ACCESSED | DIRTY),
    );
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5008,
            Reach::Store,
            Mode::Supervisor,
            0
        ),
        Ok(0x8002_0008)
    );
}

#[test]
fn a_large_page_must_be_aligned_and_an_address_must_extend_its_sign() {
    let pmp = open();
    let mut bus = bus();
    entry(&mut bus, ROOT, 1, leaf(0x8000_0000, READ | ACCESSED));
    entry(&mut bus, ROOT, 2, leaf(0x8020_0000, READ | ACCESSED));
    entry(&mut bus, ROOT, 511, leaf(0x8000_0000, READ | ACCESSED));
    let mut translation = Translation::default();
    translation.set_root((8 << 60) | (ROOT >> 12));
    let mut load = |address| {
        translation.resolve(
            &mut bus,
            &pmp,
            address,
            Reach::Load,
            Mode::Supervisor,
            0,
        )
    };

    assert_eq!(load(0x4012_3456), Ok(0x8012_3456));
    assert_eq!(load(0x8000_0000), Err(ExceptionCause::LoadPageFault));
    assert_eq!(load(0xffff_ffff_c000_0010), Ok(0x8000_0010));
    assert_eq!(
        load(0x0000_7fff_c000_0010),
        Err(ExceptionCause::LoadPageFault)
    );
}

#[test]
fn a_page_is_found_through_three_levels_and_keeps_its_offset() {
    let (mut bus, mut translation) = mapped(READ | WRITE | ACCESSED | DIRTY);
    let pmp = open();
    assert!(translation.enabled());
    for _ in 0..2 {
        assert_eq!(
            translation.resolve(
                &mut bus,
                &pmp,
                0x5abc,
                Reach::Load,
                Mode::Supervisor,
                0
            ),
            Ok(0x8002_0abc)
        );
    }
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x6000,
            Reach::Load,
            Mode::Supervisor,
            0
        ),
        Err(ExceptionCause::LoadPageFault)
    );
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5000,
            Reach::Fetch,
            Mode::Supervisor,
            0
        ),
        Err(ExceptionCause::InstructionPageFault)
    );
}

#[test]
fn a_table_the_supervisor_may_not_read_is_an_access_fault_and_a_new_root_forgets_pages()
 {
    let (mut bus, mut translation) = mapped(READ | ACCESSED);
    assert_eq!(
        translation.resolve(
            &mut bus,
            &Pmp::default(),
            0x5000,
            Reach::Store,
            Mode::Supervisor,
            0
        ),
        Err(ExceptionCause::StoreAccessFault)
    );
    let pmp = open();
    assert!(
        translation
            .resolve(&mut bus, &pmp, 0x5000, Reach::Load, Mode::Supervisor, 0)
            .is_ok()
    );
    entry(&mut bus, LAST, 5, 0);
    assert!(
        translation
            .resolve(&mut bus, &pmp, 0x5000, Reach::Load, Mode::Supervisor, 0)
            .is_ok()
    );
    translation.flush();
    assert!(
        translation
            .resolve(&mut bus, &pmp, 0x5000, Reach::Load, Mode::Supervisor, 0)
            .is_err()
    );
    translation.set_root(3 << 60);
    assert_eq!(translation.root() >> 60, 8);
    translation.set_root(0);
    assert!(!translation.enabled());
}

#[test]
fn a_user_page_is_for_user_mode_unless_the_supervisor_asks_and_never_to_execute()
 {
    let pmp = open();
    let (mut bus, mut translation) = mapped(READ | EXECUTE | USER | ACCESSED);
    let mut reach = |reach, mode, status| {
        translation
            .resolve(&mut bus, &pmp, 0x5000, reach, mode, status)
            .is_ok()
    };
    assert!(reach(Reach::Load, Mode::User, 0));
    assert!(reach(Reach::Fetch, Mode::User, 0));
    assert!(!reach(Reach::Load, Mode::Supervisor, 0));
    assert!(reach(Reach::Load, Mode::Supervisor, SUPERVISOR_USER_MEMORY));
    assert!(!reach(
        Reach::Fetch,
        Mode::Supervisor,
        SUPERVISOR_USER_MEMORY
    ));
}

#[test]
fn an_execute_only_page_is_read_only_when_the_status_says_so() {
    let pmp = open();
    let (mut bus, mut translation) = mapped(EXECUTE | ACCESSED);
    assert!(
        translation
            .resolve(&mut bus, &pmp, 0x5000, Reach::Load, Mode::Supervisor, 0)
            .is_err()
    );
    assert_eq!(
        translation.resolve(
            &mut bus,
            &pmp,
            0x5000,
            Reach::Load,
            Mode::Supervisor,
            EXECUTABLE_READABLE
        ),
        Ok(0x8002_0000)
    );
}

fn bus() -> Bus {
    crate::profile::reader::read(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cva6.toml"
    )))
    .unwrap()
    .build_supervisor_raw(0x8000_0000, &[0x13, 0, 0, 0])
    .unwrap()
    .bus
}

fn entry(bus: &mut Bus, table: u64, index: u64, value: u64) {
    bus.write(table + index * 8, 4, value as u32).unwrap();
    bus.write(table + index * 8 + 4, 4, (value >> 32) as u32)
        .unwrap();
}

fn leaf(frame: u64, flags: u64) -> u64 {
    (frame >> 12) << 10 | flags | VALID
}

fn mapped(flags: u64) -> (Bus, Translation) {
    let mut bus = bus();
    entry(&mut bus, ROOT, 0, pointer(MIDDLE));
    entry(&mut bus, MIDDLE, 0, pointer(LAST));
    entry(&mut bus, LAST, 5, leaf(0x8002_0000, flags));
    let mut translation = Translation::default();
    translation.set_root((8 << 60) | (ROOT >> 12));
    (bus, translation)
}

fn open() -> Pmp {
    let mut pmp = Pmp::default();
    pmp.set_address(0, u64::MAX);
    pmp.set_configuration(0x1f);
    pmp
}

fn pointer(table: u64) -> u64 {
    (table >> 12) << 10 | VALID
}
