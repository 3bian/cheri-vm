// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::pointer::cheriot::Capability;
use crate::machine::bus::Bus;
use crate::machine::error::EmulatorError;
use crate::machine::memory::MemoryRegion;

impl CapabilityMemory for Bus {
    fn read_capability(
        &self,
        address: u32,
    ) -> Result<Capability, EmulatorError> {
        let (bits, tag) = self.read_tagged(u64::from(address))?;
        Ok(Capability::from_memory(bits, tag))
    }

    fn write_capability(
        &mut self,
        address: u32,
        capability: Capability,
    ) -> Result<(), EmulatorError> {
        let (bits, tag) = capability.to_memory();
        self.write_tagged(u64::from(address), bits, tag)
    }
}

pub(crate) trait CapabilityMemory {
    fn read_capability(
        &self,
        address: u32,
    ) -> Result<Capability, EmulatorError>;

    fn write_capability(
        &mut self,
        address: u32,
        capability: Capability,
    ) -> Result<(), EmulatorError>;
}

impl CapabilityMemory for MemoryRegion {
    fn read_capability(
        &self,
        address: u32,
    ) -> Result<Capability, EmulatorError> {
        let (bits, tag) = self.read_tagged(u64::from(address))?;
        Ok(Capability::from_memory(bits, tag))
    }

    fn write_capability(
        &mut self,
        address: u32,
        capability: Capability,
    ) -> Result<(), EmulatorError> {
        let (bits, tag) = capability.to_memory();
        self.write_tagged(u64::from(address), bits, tag)
    }
}
