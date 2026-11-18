// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{DATA, EBREAK, csrr, load_capability, query, store_capability};
use crate::hart::execution::TrapPolicy;
use crate::hart::{Hart, Rv64Rvy};
use crate::isa::pointer::rvy::{Capability128, permissions};
use crate::machine::error::{EmulatorError, ExceptionCause};

const PROFILE: &str = "name = \"levels\"\n[emulator]\nprocessor = \
                       \"rv64\"\nextensions = [\"y\", \"zylg\", \"i\", \"m\", \
                       \"a\", \"c\", \"zicsr\", \"zifencei\"]\ntiming = \
                       \"per-step\"\nmemory = { ram = { start = 0x4000_0000, \
                       length = 0x1_0000 } }\n";

#[test]
fn a_load_without_load_global_gives_a_local_capability() {
    let global = data();
    let shallow = data().without_permissions(permissions::LOAD_GLOBAL);
    let sealed = global.with_seal(true);
    for (value, expected) in [
        (
            global,
            global.without_permissions(
                permissions::GLOBAL | permissions::LOAD_GLOBAL,
            ),
        ),
        (sealed, sealed.without_permissions(permissions::GLOBAL)),
    ] {
        let hart = finished(
            &[
                store_capability(3, 1, 0),
                load_capability(4, 2, 0),
                load_capability(5, 1, 0),
                EBREAK,
            ],
            &[(1, global), (2, shallow), (3, value)],
        );
        let loaded = hart.processor.capability(4);
        assert!(loaded.tag);
        assert_eq!(loaded, expected);
        assert_eq!(loaded.permissions() & permissions::GLOBAL, 0);
        assert_eq!(loaded.sealed(), value.sealed());
        assert_eq!(
            loaded.permissions() & permissions::LOAD_GLOBAL != 0,
            value.sealed()
        );
        assert_eq!(hart.processor.capability(5), value);
    }
}

#[test]
fn a_local_capability_is_stored_with_its_tag_only_through_store_local() {
    let global = data();
    let local = data().without_permissions(permissions::GLOBAL);
    let confined = data().without_permissions(permissions::STORE_LOCAL);
    for (authority, value, kept) in [
        (global, global, true),
        (global, local, true),
        (confined, global, true),
        (confined, local, false),
    ] {
        let hart = finished(
            &[store_capability(3, 2, 0), load_capability(4, 1, 0), EBREAK],
            &[(1, global), (2, authority), (3, value)],
        );
        assert_eq!(
            hart.processor.capability(4),
            Capability128 { tag: kept, ..value },
            "{authority:?} {value:?}"
        );
    }
}

#[test]
fn a_machine_with_levels_starts_global_and_reads_its_level_bits_back() {
    let root = Capability128::infinite(true);
    let local = data().without_permissions(permissions::GLOBAL);
    let hart = finished(
        &[query(1, 1, 6), query(1, 2, 7), csrr(1, 0x305), EBREAK],
        &[(1, data()), (2, local)],
    );
    let all =
        permissions::ALL | permissions::RESERVED_ONES | permissions::LEVELS;
    assert_eq!(hart.processor.read_register_unchecked(6), all);
    assert_eq!(
        hart.processor.read_register_unchecked(7),
        all & !permissions::GLOBAL
    );
    assert_eq!(
        hart.processor.capability(1).metadata & !0x7ff_ffff,
        root.metadata
    );
    assert_eq!(
        hart.processor.program_counter_capability().metadata,
        root.metadata
    );
}

fn data() -> Capability128 {
    Capability128::infinite(true)
        .with_address(DATA)
        .with_bounds(64, true)
}

fn finished(words: &[u32], held: &[(u8, Capability128)]) -> Hart<Rv64Rvy> {
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut hart = crate::profile::reader::read(PROFILE)
        .unwrap()
        .build_capability_raw(super::BASE, &firmware)
        .unwrap();
    hart.trap_policy = TrapPolicy::Stop;
    for (register, capability) in held {
        hart.processor.set_capability(*register, *capability);
    }
    for _ in 0..20 {
        match hart.step() {
            Ok(_) => (),
            Err(EmulatorError::Trap {
                cause: ExceptionCause::Breakpoint,
                ..
            }) => return hart,
            Err(error) => panic!("{error}"),
        }
    }
    panic!("the program did not finish");
}
