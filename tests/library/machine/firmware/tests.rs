// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::ExtensionSet;
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::firmware::*;
use crate::machine::memory::{MemoryKind, MemoryRegion, Tagging};

#[test]
fn a_64_bit_or_big_endian_elf_is_refused_by_name() {
    let mut bus = Bus::new();
    let profile = ExtensionSet::I.with(ExtensionSet::ZICSR);
    for ident in [&b"\x7fELF\x02\x01\x01"[..], &b"\x7fELF\x01\x02\x01"[..]] {
        let mut image = [0_u8; 64];
        image[..7].copy_from_slice(ident);
        assert_eq!(
            load_elf(&mut bus, &image, profile).unwrap_err().to_string(),
            EmulatorError::Firmware(REQUIRED_KIND.into()).to_string()
        );
    }
}

#[test]
fn malformed_elf_is_rejected_without_panicking() {
    let mut bus = Bus::new();
    bus.add_memory(
        MemoryRegion::new(
            "ram".into(),
            0,
            4096,
            MemoryKind::Ram,
            Tagging::None,
        )
        .unwrap(),
    )
    .unwrap();
    let profile = ExtensionSet::I.with(ExtensionSet::ZICSR);

    for bytes in [&[][..], &b"\x7fELF"[..], &[0_u8; 52][..]] {
        assert!(load_elf(&mut bus, bytes, profile).is_err());
    }
}
