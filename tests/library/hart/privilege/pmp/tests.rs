// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::privilege::pmp::Pmp;
use crate::hart::privilege::{Mode, Reach};

const EXECUTE: u64 = 4;
const FOUR_BYTES: u64 = 2 << 3;
const LOCKED: u64 = 1 << 7;
const POWER_OF_TWO: u64 = 3 << 3;
const READ: u64 = 1;
const TOP_OF_RANGE: u64 = 1 << 3;
const WRITE: u64 = 2;

#[test]
fn a_register_keeps_54_bits_and_write_without_read_is_not_kept() {
    let mut pmp = Pmp::default();
    pmp.set_address(0, u64::MAX);
    assert_eq!(pmp.address(0), (1 << 54) - 1);
    assert_eq!(pmp.address(8), 0);
    pmp.set_configuration(WRITE | POWER_OF_TWO | (1 << 5));
    assert_eq!(pmp.configuration(), POWER_OF_TWO);
}

#[test]
fn an_access_of_no_bytes_is_judged_by_its_address() {
    let mut pmp = Pmp::default();
    pmp.set_address(0, 0x8000_0000 >> 2);
    pmp.set_configuration(FOUR_BYTES | READ);
    assert!(pmp.allows(0x8000_0000, 0, Reach::Load, Mode::User));
    assert!(!pmp.allows(0x8000_0004, 0, Reach::Load, Mode::User));
    assert!(pmp.allows(0, 0, Reach::Load, Mode::Machine));
}

#[test]
fn each_way_of_matching_covers_the_range_it_names() {
    let mut pmp = Pmp::default();
    pmp.set_address(0, 0x8000_0000 >> 2);
    pmp.set_address(1, 0x8000_1000 >> 2);
    pmp.set_address(2, 0x9000_0000 >> 2);
    pmp.set_address(3, (0xa000_0000 >> 2) | 0x1ff);
    pmp.set_configuration(
        ((TOP_OF_RANGE | READ) << 8)
            | ((FOUR_BYTES | READ | WRITE) << 16)
            | ((POWER_OF_TWO | READ | EXECUTE) << 24),
    );
    let user =
        |address, width, reach| pmp.allows(address, width, reach, Mode::User);

    assert!(user(0x8000_0000, 4, Reach::Load));
    assert!(user(0x8000_0ffc, 4, Reach::Load));
    assert!(!user(0x8000_1000, 4, Reach::Load));
    assert!(!user(0x8000_0000, 4, Reach::Store));

    assert!(user(0x9000_0000, 4, Reach::Store));
    assert!(!user(0x9000_0004, 4, Reach::Store));
    assert!(!user(0x9000_0000, 8, Reach::Load));

    assert!(user(0xa000_0000, 2, Reach::Fetch));
    assert!(user(0xa000_0ffe, 2, Reach::Fetch));
    assert!(!user(0xa000_1000, 2, Reach::Fetch));
    assert!(!user(0xa000_0000, 4, Reach::Store));
}

#[test]
fn the_lowest_entry_that_matches_decides_and_a_locked_one_binds_machine_mode() {
    let mut pmp = Pmp::default();
    pmp.set_address(0, 0x8000_0000 >> 2);
    pmp.set_address(1, u64::MAX);
    pmp.set_configuration(
        (FOUR_BYTES | READ | LOCKED)
            | ((POWER_OF_TWO | READ | WRITE | EXECUTE) << 8),
    );
    assert!(pmp.allows(0x8000_0000, 4, Reach::Load, Mode::Machine));
    assert!(!pmp.allows(0x8000_0000, 4, Reach::Store, Mode::Machine));
    assert!(!pmp.allows(0x8000_0000, 4, Reach::Store, Mode::Supervisor));
    assert!(pmp.allows(0x8000_0004, 4, Reach::Store, Mode::Supervisor));
    assert!(pmp.allows(0xffff_ffff_fff8, 8, Reach::Fetch, Mode::User));

    pmp.set_address(0, 0);
    pmp.set_configuration(0);
    assert_eq!(pmp.address(0), 0x8000_0000 >> 2);
    assert_eq!(pmp.configuration() & 0xff, FOUR_BYTES | READ | LOCKED);
    assert_eq!(pmp.configuration() >> 8, 0);
}

#[test]
fn with_no_entry_machine_mode_reaches_everything_and_the_lower_modes_nothing() {
    let pmp = Pmp::default();
    assert!(pmp.allows(0x8000_0000, 8, Reach::Store, Mode::Machine));
    assert!(!pmp.allows(0x8000_0000, 4, Reach::Load, Mode::Supervisor));
    assert!(!pmp.allows(0x8000_0000, 2, Reach::Fetch, Mode::User));
}
