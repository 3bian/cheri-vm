// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::elf::{self, Elf};

const HELLO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/cheriot/hello_world.elf"
));

fn read_all(bytes: &[u8]) {
    let Ok(file) = Elf::parse(bytes) else {
        return;
    };
    let _ = file.segments();
    let _ = file.symbols();
    for section in file.sections().unwrap_or_default() {
        let _ = (section.data(), section.is_code(), section.is_compressed());
    }
}

#[test]
fn a_32_bit_file_gives_its_header_segments_sections_and_symbols() {
    let file = Elf::parse(HELLO).unwrap();
    assert!(!file.wide);
    assert_eq!(file.kind, elf::EXECUTABLE);
    assert_eq!(file.machine, elf::RISCV);
    assert_eq!(file.version, 1);
    assert_eq!(file.header_size, 52);
    assert!(file.has_program_headers());
    let segments = file.segments().unwrap();
    let loaded: Vec<_> = segments
        .iter()
        .filter(|segment| segment.kind == elf::SEGMENT_LOAD)
        .collect();
    assert!(!loaded.is_empty());
    assert!(loaded.iter().any(|segment| {
        segment.flags & elf::SEGMENT_EXECUTABLE != 0
            && (segment.virtual_address
                ..segment.virtual_address + segment.memory_size)
                .contains(&file.entry)
    }));
    let sections = file.sections().unwrap();
    assert_eq!(sections[0].name, "");
    assert!(sections.iter().any(|section| section.is_code()));
    assert!(sections.iter().any(|section| section.name == ".symtab"));
    assert!(sections.iter().all(|section| section.data().is_some()));
    let symbols = file.symbols().unwrap().unwrap();
    let hello = symbols
        .iter()
        .find(|symbol| symbol.name == "_Z9say_hellov")
        .unwrap();
    assert!(hello.is_function());
    assert_ne!(hello.section, elf::UNDEFINED);
    assert!(sections.iter().any(|section| {
        section.is_code()
            && (section.address..section.address + section.size)
                .contains(&hello.value)
    }));
}

#[test]
fn a_64_bit_header_is_read_at_its_own_offsets() {
    let mut bytes = vec![0_u8; 64];
    bytes[..4].copy_from_slice(elf::MAGIC);
    bytes[4] = elf::CLASS_64;
    bytes[5] = elf::LITTLE_ENDIAN;
    bytes[16] = 2;
    bytes[18] = 243;
    bytes[20] = 1;
    bytes[24..32].copy_from_slice(&0x8000_0000_1234_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&5_u32.to_le_bytes());
    bytes[52] = 64;
    let file = Elf::parse(&bytes).unwrap();
    assert!(file.wide);
    assert_eq!(file.entry, 0x8000_0000_1234);
    assert_eq!(file.flags, 5);
    assert_eq!(file.header_size, 64);
    assert!(!file.has_program_headers());
    assert!(file.segments().unwrap().is_empty());
    assert!(file.sections().unwrap().is_empty());
    assert!(file.symbols().unwrap().is_none());
}

#[test]
fn a_file_cut_short_anywhere_is_an_error_or_reads_less_and_never_panics() {
    for length in (0..HELLO.len()).step_by(7).chain(0..80) {
        read_all(&HELLO[..length]);
    }
    assert!(Elf::parse(&HELLO[..51]).is_err());
    let file = Elf::parse(&HELLO[..52]).unwrap();
    assert!(file.segments().is_err());
    assert!(file.sections().is_err());
}

#[test]
fn a_file_with_any_header_or_table_byte_changed_never_panics() {
    let file = Elf::parse(HELLO).unwrap();
    let tables: Vec<usize> = file
        .sections()
        .unwrap()
        .iter()
        .filter(|section| section.kind == elf::SECTION_SYMBOLS)
        .flat_map(|section| {
            let start = section.offset as usize;
            start..start + 64
        })
        .collect();
    let section_table = u32::from_le_bytes(HELLO[32..36].try_into().unwrap());
    let headers =
        section_table as usize..HELLO.len().min(section_table as usize + 400);
    for index in (0..84).chain(headers).chain(tables) {
        for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
            let mut bytes = HELLO.to_vec();
            if let Some(byte) = bytes.get_mut(index) {
                *byte = value;
            }
            read_all(&bytes);
        }
    }
}

#[test]
fn what_is_not_a_little_endian_elf_file_is_refused() {
    assert_eq!(Elf::parse(b"").err().unwrap(), "not an ELF file");
    assert_eq!(Elf::parse(b"\x7fELG").err().unwrap(), "not an ELF file");
    let mut big = HELLO.to_vec();
    big[5] = 2;
    assert_eq!(
        Elf::parse(&big).err().unwrap(),
        "unsupported ELF class or byte order"
    );
    let mut class = HELLO.to_vec();
    class[4] = 3;
    assert!(Elf::parse(&class).is_err());
    let mut entries = HELLO.to_vec();
    entries[46] = 41;
    assert_eq!(
        Elf::parse(&entries).unwrap().sections().err().unwrap(),
        "an ELF table has entries of the wrong size"
    );
}
