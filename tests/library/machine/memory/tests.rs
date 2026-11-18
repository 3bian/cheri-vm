// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e::CapabilityMemory;
use crate::isa::pointer::cheriot::Capability;
use crate::machine::EmulatorError;
use crate::machine::memory::cheriot_tags::CheriotTags;
use crate::machine::memory::*;

const ROOT: Capability = Capability::from_memory(0x7e3e_0000_0000_0000, true);

impl MemoryRegion {
    fn mark_tag_for_test(&mut self, address: u32) {
        if let Some(tags) = &mut self.tags {
            tags.set((u64::from(address) - self.base) as usize, true);
        }
    }
}

impl MemoryRegion {
    pub(crate) fn inspect_capability(
        &self,
        address: u32,
    ) -> Result<Capability, EmulatorError> {
        self.read_capability(address)
    }

    pub(crate) fn install_firmware(
        &mut self,
        address: u32,
        bytes: &[u8],
    ) -> Result<(), EmulatorError> {
        self.install_firmware_segment(address.into(), bytes, bytes.len())
    }

    pub(crate) fn tag_storage_bytes(&self) -> usize {
        self.tags.as_ref().map_or(0, CheriotTags::storage_bytes)
    }
}

#[test]
fn a_region_holds_up_to_a_gibibyte() {
    let region = |size| {
        MemoryRegion::new("ram".into(), 0, size, MemoryKind::Ram, Tagging::None)
    };
    let mut largest = region(1 << 30).unwrap();
    largest.write((1 << 30) - 4, 4, 0x1234_5678).unwrap();
    assert_eq!(largest.read((1 << 30) - 4, 4).unwrap(), 0x1234_5678);
    for size in [0, (1 << 30) + 1] {
        let message = region(size).err().unwrap().to_string();
        assert!(message.contains("from 1 byte to 1 GiB"), "{message}");
    }
}

#[test]
fn all_firmware_write_ranges_clear_only_overlapping_tags() {
    for offset in 0..=136 {
        for length in 0..=136 - offset {
            let mut memory = MemoryRegion::new(
                "ram".into(),
                0x1000,
                136,
                MemoryKind::Ram,
                Tagging::Cheriot,
            )
            .unwrap();

            for address in (0x1000..0x1088).step_by(8) {
                memory.write_capability(address, ROOT).unwrap();
            }

            memory
                .install_firmware(0x1000 + offset, &vec![0; length as usize])
                .unwrap();

            for granule in (0..136).step_by(8) {
                let touched = (offset..offset + length)
                    .any(|byte| (granule..granule + 8).contains(&byte));
                assert_eq!(
                    memory.tag_at((0x1000 + granule).into()),
                    Some(!touched),
                    "offset {offset}, length {length}, granule {granule}"
                );
                assert_eq!(
                    memory.read_capability(0x1000 + granule).unwrap().tag(),
                    !touched
                );
            }
        }
    }
}

#[test]
fn capability_and_integer_writes_preserve_neighbours_at_bitmap_boundaries() {
    let mut memory = MemoryRegion::new(
        "ram".into(),
        0,
        136,
        MemoryKind::Ram,
        Tagging::Cheriot,
    )
    .unwrap();

    for address in (0..136).step_by(8) {
        memory.write_capability(address, ROOT).unwrap();
    }

    memory.write(63, 2, 0).unwrap();
    memory.write_capability(128, ROOT.clear_tag()).unwrap();

    for address in (0..136).step_by(8) {
        assert_eq!(
            memory.tag_at(address),
            Some(!matches!(address, 56 | 64 | 128))
        );
    }

    memory.write_capability(64, ROOT).unwrap();
    assert_eq!(memory.tag_at(56), Some(false));
    assert_eq!(memory.tag_at(64), Some(true));
    assert_eq!(memory.tag_at(72), Some(true));
    assert_eq!(memory.tag_at(127), Some(true));
    assert_eq!(memory.tag_at(128), Some(false));
    assert_eq!(memory.tag_at(136), None);
}

#[test]
fn invalid_writes_preserve_existing_payload_and_tags() {
    let mut memory = MemoryRegion::new(
        "ram".into(),
        0,
        8,
        MemoryKind::Ram,
        Tagging::Cheriot,
    )
    .unwrap();
    memory.write_capability(0, ROOT).unwrap();
    assert!(memory.write(7, 2, 0).is_err());
    assert!(memory.write(0, 8, 0).is_err());
    assert!(memory.write_capability(1, ROOT.clear_tag()).is_err());
    assert!(memory.install_firmware(7, &[0, 0]).is_err());
    assert_eq!(memory.read_capability(0).unwrap(), ROOT);

    memory.kind = MemoryKind::Rom;
    assert!(memory.write(0, 1, 0).is_err());
    assert!(memory.write_capability(0, ROOT.clear_tag()).is_err());
    assert_eq!(memory.read_capability(0).unwrap(), ROOT);
}

#[test]
fn memory_is_little_endian_and_rom_refuses_writes() {
    let mut memory = MemoryRegion::new(
        "rom".into(),
        0x1000,
        16,
        MemoryKind::Rom,
        Tagging::None,
    )
    .unwrap();
    memory
        .install_firmware(0x1000, &[0x78, 0x56, 0x34, 0x12])
        .unwrap();
    assert_eq!(memory.read(0x1000, 4).unwrap(), 0x12345678);
    assert!(memory.write(0x1000, 1, 1).is_err());
    assert!(memory.read(0x100f, 4).is_err());
}

#[test]
fn ordinary_and_dma_writes_clear_tags() {
    let mut memory = MemoryRegion::new(
        "ram".into(),
        0,
        32,
        MemoryKind::Ram,
        Tagging::Cheriot,
    )
    .unwrap();
    memory.mark_tag_for_test(8);
    memory.mark_tag_for_test(16);
    memory.write(15, 2, 42).unwrap();
    assert_eq!(memory.tag_at(8), Some(false));
    assert_eq!(memory.tag_at(16), Some(false));
}

#[test]
fn ordinary_bytes_cannot_create_tags_and_untagged_memory_cannot_preserve_them()
{
    for tagging in [Tagging::Cheriot, Tagging::None] {
        let mut memory =
            MemoryRegion::new("ram".into(), 0, 8, MemoryKind::Ram, tagging)
                .unwrap();
        memory
            .install_firmware(0, &ROOT.to_memory().0.to_le_bytes())
            .unwrap();
        assert!(!memory.read_capability(0).unwrap().tag());
        memory.write_capability(0, ROOT).unwrap();
        assert_eq!(
            memory.read_capability(0).unwrap().tag(),
            tagging == Tagging::Cheriot
        );
        memory.write(0, 4, 0).unwrap();
        assert!(!memory.read_capability(0).unwrap().tag());
    }
}

#[test]
fn tag_storage_is_one_bit_per_eight_guest_bytes_including_partial_bytes() {
    for (size, expected) in [
        (8, 1),
        (64, 1),
        (72, 2),
        (136, 3),
        (64 * 1024 * 1024, 1024 * 1024),
    ] {
        let memory = MemoryRegion::new(
            "ram".into(),
            0,
            size,
            MemoryKind::Ram,
            Tagging::Cheriot,
        )
        .unwrap();
        assert_eq!(memory.tag_storage_bytes(), expected);
        assert_eq!(memory.tag_at(0), Some(false));
        assert_eq!(memory.tag_at((size as u32 - 1).into()), Some(false));
    }

    let memory =
        MemoryRegion::new("ram".into(), 0, 8, MemoryKind::Ram, Tagging::None)
            .unwrap();
    assert_eq!(memory.tag_storage_bytes(), 0);
    assert_eq!(memory.tag_at(0), None);
}
