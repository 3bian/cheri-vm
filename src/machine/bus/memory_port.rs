// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::memory::MemoryRegion;

pub(crate) struct MemoryPort<'a> {
    memory: &'a mut [MemoryRegion],
}

impl<'a> MemoryPort<'a> {
    pub(crate) const GRANULE_BYTES: u32 = 8;

    pub(crate) fn clear_tag(&mut self, address: u64) -> bool {
        self.memory
            .iter_mut()
            .find(|region| region.contains(address, 1))
            .is_some_and(|region| region.clear_tag(address))
    }

    pub(super) fn new(memory: &'a mut [MemoryRegion]) -> Self {
        Self { memory }
    }

    pub(crate) fn read_byte(&self, address: u64) -> Option<u8> {
        let region = self
            .memory
            .iter()
            .find(|region| region.contains(address, 1))?;
        region.read(address, 1).ok().map(|value| value as u8)
    }

    pub(crate) fn read_granule(&self, address: u64) -> Option<(u64, bool)> {
        let region = self.memory.iter().find(|region| {
            region.contains(address, Self::GRANULE_BYTES as usize)
        })?;
        region.read_tagged(address).ok()
    }
}
