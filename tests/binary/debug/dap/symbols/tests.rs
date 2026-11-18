// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::debug::dap::symbols::Symbols;

const COMPRESSED: u32 = 0x800;

#[test]
fn a_compressed_debug_section_is_noticed() {
    assert!(Symbols::load(&elf_with_debug_line(COMPRESSED)).compressed());
}

#[test]
fn a_plain_debug_section_is_not_called_compressed() {
    assert!(!Symbols::load(&elf_with_debug_line(0)).compressed());
}

fn elf_with_debug_line(flags: u32) -> Vec<u8> {
    let names = b"\0.shstrtab\0.debug_line\0";
    let mut image = vec![0_u8; 0x200];
    image[..7].copy_from_slice(b"\x7fELF\x01\x01\x01");
    for (offset, value) in
        [(16, 2_u16), (18, 243), (40, 52), (46, 40), (48, 3), (50, 1)]
    {
        image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    for (offset, value) in [(20, 1_u32), (32, 0x100)] {
        image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    image[0x40..0x40 + names.len()].copy_from_slice(names);
    let sections = [
        [1_u32, 3, 0, 0, 0x40, names.len() as u32, 0, 0, 1, 0],
        [11, 1, flags, 0, 0x80, 16, 0, 0, 1, 0],
    ];
    for (index, section) in sections.iter().enumerate() {
        for (field, value) in section.iter().enumerate() {
            let offset = 0x100 + (index + 1) * 40 + field * 4;
            image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    image[0x80] = 1;
    image
}
