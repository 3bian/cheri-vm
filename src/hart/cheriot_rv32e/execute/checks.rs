// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e::{CapabilityFaultCause, CheriotRv32e};
use crate::hart::{Fault, Hart};
use crate::isa::pointer::cheriot::permissions;
use crate::isa::pointer::{Access, Cheriot, PointerModel, Violation};

impl Hart<CheriotRv32e> {
    pub(super) fn check_csr_access(
        address: u16,
        writing: bool,
    ) -> Result<(), Fault> {
        let defined = matches!(
            address,
            0x300 | 0x301 | 0x304 | 0x305 | 0x310 | 0x320 | 0x340..=0x344
                | 0x7a0 | 0xb00 | 0xb02 | 0xb80 | 0xb82 | 0xbc0..=0xbc2
                | 0xf11..=0xf14
        );
        if !defined || (writing && address & 0xc00 == 0xc00) {
            return Err(Fault::illegal());
        }
        Ok(())
    }

    pub(super) fn check_memory_access(
        &self,
        register: u8,
        address: u32,
        width: usize,
        access: Access,
    ) -> Result<u16, Fault> {
        let authority = self.processor.read_register(register)?;
        Cheriot::check_access(authority, address, width, access).map_err(
            |violation| {
                let cause = match violation {
                    Violation::Alignment => {
                        return Fault {
                            cause: if access.writes() { 6 } else { 4 },
                            value: address,
                        };
                    }
                    Violation::Bounds => CapabilityFaultCause::Bounds,
                    Violation::CapabilityPermission => {
                        CapabilityFaultCause::StoreCapabilityPermission
                    }
                    Violation::Permission if access.writes() => {
                        CapabilityFaultCause::StorePermission
                    }
                    Violation::Permission => {
                        CapabilityFaultCause::LoadPermission
                    }
                    Violation::Seal => CapabilityFaultCause::Seal,
                    Violation::Tag => CapabilityFaultCause::Tag,
                };
                Fault::capability_cause(cause, register)
            },
        )?;
        Ok(authority.permissions())
    }

    pub(super) fn require_system_registers(
        &self,
        register: u8,
    ) -> Result<(), Fault> {
        if self.processor.pcc.permissions() & permissions::SYSTEM_REGISTERS == 0
        {
            return Err(Fault::capability_cause(
                CapabilityFaultCause::SystemRegisterPermission,
                register,
            ));
        }
        Ok(())
    }
}
