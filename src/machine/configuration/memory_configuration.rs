// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e;
use crate::machine::EmulatorError;
use crate::machine::memory::Tagging;

impl super::MachineConfiguration {
    pub(super) fn validate_cheriot_memory(&self) -> Result<(), EmulatorError> {
        const MEMORY_BASE: u64 =
            cheriot_rv32e::Processor::REVOCABLE_MEMORY_BASE as u64;
        const REVOCATION_BASE: u64 =
            cheriot_rv32e::Processor::REVOCATION_BITMAP_BASE as u64;

        let mut highest_address = MEMORY_BASE;
        for region in self.memory.iter().filter(|region| region.kind == "ram") {
            if region.tagging != Tagging::Cheriot || region.base < MEMORY_BASE {
                return Err(EmulatorError::Configuration(
                    "CHERIoT RAM must be tagged and start at or above \
                     0x80000000"
                        .into(),
                ));
            }
            let end = region.base.checked_add(region.size as u64).ok_or_else(
                || EmulatorError::Configuration("RAM address overflow".into()),
            )?;
            highest_address = highest_address.max(end);
        }

        let bitmap_size = (highest_address - MEMORY_BASE).div_ceil(64);
        let bitmap_end = REVOCATION_BASE + bitmap_size;
        let bitmap_is_mapped = self.memory.iter().any(|region| {
            region.kind == "ram"
                && region.base <= REVOCATION_BASE
                && region.base.saturating_add(region.size as u64) >= bitmap_end
        });
        if !bitmap_is_mapped {
            return Err(EmulatorError::Configuration(
                "CHERIoT RAM must cover the revocation bitmap at 0x83000000"
                    .into(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryConfiguration {
    pub base: u64,
    pub executable: bool,
    pub kind: String,
    pub name: String,
    pub size: usize,
    pub tagging: Tagging,
}
