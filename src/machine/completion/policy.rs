// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::bus::Bus;
use crate::machine::error::EmulatorError;
use crate::machine::firmware::elf_symbol_address;

use super::HostInterface;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionPolicy {
    Address(u64),
    Disabled,
    ElfSymbol,
}

impl CompletionPolicy {
    pub(crate) fn build(
        self,
        bus: &Bus,
        firmware: Option<&[u8]>,
    ) -> Result<Option<HostInterface>, EmulatorError> {
        let address = match self {
            Self::Address(address) => address,
            Self::Disabled => return Ok(None),
            Self::ElfSymbol => elf_symbol_address(
                firmware.ok_or_else(|| {
                    EmulatorError::Configuration(
                        "ELF-symbol completion requires ELF firmware".into(),
                    )
                })?,
                "tohost",
            )?,
        };
        HostInterface::on_bus(bus, address).map(Some)
    }
}
