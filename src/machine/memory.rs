// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::EmulatorError;

pub(crate) mod cheriot_tags;
mod memory_kind;
pub(crate) mod tag_bitmap;
mod tagging;

pub(crate) use memory_kind::MemoryKind;
pub(crate) use tagging::Tagging;

use cheriot_tags::CheriotTags;

pub(crate) const BACKING_LIMIT: usize = 2 << 30;
pub(crate) const REGION_LIMIT: usize = 1 << 30;

pub(crate) struct MemoryRegion {
    pub base: u64,
    bytes: Vec<u8>,
    pub executable: bool,
    pub kind: MemoryKind,
    pub name: String,
    pub(crate) tags: Option<CheriotTags>,
}

impl MemoryRegion {
    pub(crate) fn clear_tag(&mut self, address: u64) -> bool {
        if !self.contains(address, 1) {
            return false;
        }
        let offset = (address - self.base) as usize;
        match &mut self.tags {
            Some(tags) if tags.get(offset) => {
                tags.set(offset, false);
                true
            }
            _ => false,
        }
    }

    fn clear_tags(&mut self, offset: usize, size: usize) {
        if let Some(tags) = &mut self.tags {
            tags.clear(offset, size);
        }
    }

    #[inline]
    pub(crate) fn contains(&self, address: u64, width: usize) -> bool {
        let start = address;
        start >= self.base
            && width as u64
                <= (self.base + self.bytes.len() as u64).saturating_sub(start)
            && start <= self.base + self.bytes.len() as u64
    }

    pub(crate) fn install_firmware_segment(
        &mut self,
        address: u64,
        bytes: &[u8],
        memory_size: usize,
    ) -> Result<(), EmulatorError> {
        if bytes.len() > memory_size {
            return Err(EmulatorError::Firmware(
                "firmware segment data exceeds its memory size".into(),
            ));
        }
        let offset = self.offset(address, memory_size)?;
        self.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
        self.bytes[offset + bytes.len()..offset + memory_size].fill(0);
        self.clear_tags(offset, memory_size);

        Ok(())
    }

    pub(crate) fn new(
        name: String,
        base: u64,
        size: usize,
        kind: MemoryKind,
        tagging: Tagging,
    ) -> Result<Self, EmulatorError> {
        if size == 0 || size > REGION_LIMIT {
            return Err(EmulatorError::Configuration(format!(
                "memory region {name} must have size from 1 byte to 1 GiB"
            )));
        }

        if let Some(granule) = tagging.granule_bytes()
            && (!(base as usize).is_multiple_of(granule)
                || !size.is_multiple_of(granule))
        {
            return Err(EmulatorError::Configuration(format!(
                "tagged memory region {name} must be {granule}-byte aligned \
                 and sized"
            )));
        }

        let tags = match tagging {
            Tagging::Cheriot => Some(CheriotTags::new(size)),
            Tagging::None => None,
            Tagging::Rvy => Some(CheriotTags::with_granule(size, 16)),
        };
        Ok(Self {
            name,
            base,
            kind,
            executable: true,
            bytes: vec![0; size],
            tags,
        })
    }

    #[inline]
    fn offset(
        &self,
        address: u64,
        width: usize,
    ) -> Result<usize, EmulatorError> {
        if !self.contains(address, width) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "access outside memory region".to_string(),
            });
        }

        Ok((address - self.base) as usize)
    }

    #[inline]
    pub(crate) fn read(
        &self,
        address: u64,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        if !matches!(width, 1 | 2 | 4) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "unsupported width".into(),
            });
        }

        let offset = self.offset(address, width)?;
        let bytes = &self.bytes[offset..offset + width];
        Ok(match width {
            1 => u32::from(bytes[0]),
            2 => u32::from(u16::from_le_bytes([bytes[0], bytes[1]])),
            _ => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    pub(crate) fn read_tagged(
        &self,
        address: u64,
    ) -> Result<(u64, bool), EmulatorError> {
        let offset = self.offset(address, 8)?;

        if !address.is_multiple_of(8) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "capability address must be eight-byte aligned"
                    .into(),
            });
        }

        let mut bytes = [0; 8];
        bytes.copy_from_slice(&self.bytes[offset..offset + 8]);

        Ok((
            u64::from_le_bytes(bytes),
            self.tag_at(address).unwrap_or(false),
        ))
    }

    pub(crate) fn set_tag(&mut self, address: u64) {
        if self.contains(address, 1)
            && self.kind != MemoryKind::Rom
            && let Some(tags) = &mut self.tags
        {
            tags.set((address - self.base) as usize, true);
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn tag_at(&self, address: u64) -> Option<bool> {
        if !self.contains(address, 1) {
            return None;
        }

        self.tags
            .as_ref()
            .map(|tags| tags.get((address - self.base) as usize))
    }

    pub(crate) fn tagging(&self) -> Tagging {
        match &self.tags {
            Some(tags) if tags.granule == 16 => Tagging::Rvy,
            Some(_) => Tagging::Cheriot,
            None => Tagging::None,
        }
    }

    pub(crate) fn write(
        &mut self,
        address: u64,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        if self.kind == MemoryKind::Rom {
            return Err(EmulatorError::Bus {
                address,
                explanation: "write to ROM".into(),
            });
        }

        if !matches!(width, 1 | 2 | 4) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "unsupported width".into(),
            });
        }

        let offset = self.offset(address, width)?;

        for index in 0..width {
            self.bytes[offset + index] = (value >> (8 * index)) as u8;
        }

        self.clear_tags(offset, width);

        Ok(())
    }

    pub(crate) fn write_tagged(
        &mut self,
        address: u64,
        bits: u64,
        tag: bool,
    ) -> Result<(), EmulatorError> {
        let offset = self.offset(address, 8)?;

        if self.kind == MemoryKind::Rom || !address.is_multiple_of(8) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "capability write requires aligned RAM".into(),
            });
        }

        self.bytes[offset..offset + 8].copy_from_slice(&bits.to_le_bytes());

        if let Some(tags) = &mut self.tags {
            tags.set(offset, tag);
        }

        Ok(())
    }
}
