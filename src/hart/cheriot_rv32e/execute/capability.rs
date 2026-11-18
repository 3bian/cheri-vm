// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[cfg(feature = "verification")]
use crate::hart::cheriot_rv32e::MemoryAccess;
use crate::hart::cheriot_rv32e::{
    CapabilityFaultCause, CapabilityMemory, CheriotRv32e,
};
use crate::hart::{Fault, Hart};
use crate::isa::extension::cheriot::{CapabilityInstruction, Operation};
use crate::isa::pointer::Access;
use crate::isa::pointer::cheriot::{Capability, permissions};

impl Hart<CheriotRv32e> {
    pub(super) fn capability(
        &mut self,
        decoded: CapabilityInstruction,
        next: u32,
    ) -> Result<(), Fault> {
        let first = self.processor.read_register(decoded.first)?;
        let second = if decoded.operation == Operation::SpecialReadWrite {
            Capability::integer(0)
        } else {
            self.processor.read_register(decoded.second)?
        };
        let value = second.address();
        let immediate = decoded.immediate as u32;
        let destination = decoded.destination;
        let capability = match decoded.operation {
            Operation::AddressFromGlobal
            | Operation::IncrementAddressImmediate => {
                first.set_address(first.address().wrapping_add(immediate))
            }
            Operation::AddressFromPc => self.processor.pcc.set_address(
                self.processor.program_counter.wrapping_add(immediate),
            ),
            Operation::AndPermissions => {
                first.restrict_permissions(value as u16)
            }
            Operation::ClearTag => first.clear_tag(),
            Operation::GetAddress => Capability::integer(first.address()),
            Operation::GetBase => Capability::integer(first.bounds().0),
            Operation::GetHigh => {
                Capability::integer((first.to_memory().0 >> 32) as u32)
            }
            Operation::GetLength => Capability::integer(first.length()),
            Operation::GetPermissions => {
                Capability::integer(u32::from(first.permissions()))
            }
            Operation::GetTag => Capability::integer(u32::from(first.tag())),
            Operation::GetTop => Capability::integer(
                first.bounds().1.min(u64::from(u32::MAX)) as u32,
            ),
            Operation::GetType => {
                Capability::integer(u32::from(first.object_type()))
            }
            Operation::IncrementAddress => {
                first.set_address(first.address().wrapping_add(value))
            }
            Operation::Jump => {
                self.link(destination, next)?;
                self.processor.program_counter =
                    self.processor.program_counter.wrapping_add(immediate);
                return Ok(());
            }
            Operation::JumpRegister => {
                self.jump_register(decoded, first, next)?;
                return Ok(());
            }
            Operation::Load => {
                self.load_capability(decoded.first, first, immediate)?
            }
            Operation::Move => first,
            Operation::RepresentableAlignment => {
                Capability::integer(Capability::alignment_mask(first.address()))
            }
            Operation::RepresentableLength => {
                Capability::integer(Capability::rounded_length(first.address()))
            }
            Operation::Seal => first.seal(second),
            Operation::SetAddress => first.set_address(value),
            Operation::SetBounds => first.set_bounds(value, false).0,
            Operation::SetBoundsExact => first.set_bounds(value, true).0,
            Operation::SetBoundsImmediate => {
                first.set_bounds(immediate, false).0
            }
            Operation::SetBoundsRoundDown => first.set_bounds_round_down(value),
            Operation::SetHigh => Capability::from_memory(
                (u64::from(value) << 32) | u64::from(first.address()),
                false,
            ),
            Operation::SpecialReadWrite => {
                self.special(decoded.second, decoded.first, first)?
            }
            Operation::Store => {
                self.store_capability(decoded.first, first, second, immediate)?;
                self.processor.program_counter = next;
                return Ok(());
            }
            Operation::Subtract => {
                Capability::integer(first.address().wrapping_sub(value))
            }
            Operation::TestEqual => {
                Capability::integer(u32::from(first == second))
            }
            Operation::TestSubset => {
                Capability::integer(u32::from(first.subset_contains(second)))
            }
            Operation::Unseal => first.unseal(second),
        };
        self.processor.write_register(destination, capability)?;
        self.processor.program_counter = next;

        Ok(())
    }

    fn jump_register(
        &mut self,
        decoded: CapabilityInstruction,
        first: Capability,
        next: u32,
    ) -> Result<(), Fault> {
        let destination = decoded.destination;
        let immediate = decoded.immediate as u32;
        let object_type = first.object_type();

        if !first.tag() {
            return Err(Fault::capability_cause(
                CapabilityFaultCause::Tag,
                decoded.first,
            ));
        }

        let legal = (destination == 0
            && decoded.first == 1
            && matches!(object_type, 4 | 5))
            || (destination == 0
                && decoded.first != 1
                && matches!(object_type, 0 | 1))
            || (destination != 0 && matches!(object_type, 0 | 1))
            || (destination == 1 && matches!(object_type, 0..=3));

        if !legal || (object_type != 0 && immediate != 0) {
            return Err(Fault::capability_cause(
                CapabilityFaultCause::Seal,
                decoded.first,
            ));
        }

        if first.permissions() & permissions::EXECUTE == 0 {
            return Err(Fault::capability_cause(
                CapabilityFaultCause::ExecutePermission,
                decoded.first,
            ));
        }

        self.link(destination, next)?;
        self.processor.pcc = Capability::from_memory(
            first.to_memory().0 & !(7u64 << 54),
            first.tag(),
        );

        if matches!(object_type, 2 | 4) {
            self.processor.machine_registers.machine_status &= !8;
        }

        if matches!(object_type, 3 | 5) {
            self.processor.machine_registers.machine_status |= 8;
        }

        self.processor.program_counter =
            first.address().wrapping_add(immediate) & !1;
        Ok(())
    }

    pub(super) fn link(
        &mut self,
        destination: u8,
        address: u32,
    ) -> Result<(), Fault> {
        let mut link = self.processor.pcc.set_address(address);
        if destination == 1 {
            let object_type =
                if self.processor.machine_registers.machine_status & 8 != 0 {
                    5
                } else {
                    4
                };
            link = Capability::from_memory(
                (link.to_memory().0 & !(7u64 << 54)) | (object_type << 54),
                link.tag(),
            );
        }
        self.processor.write_register(destination, link)
    }

    fn load_capability(
        &mut self,
        authority: u8,
        first: Capability,
        immediate: u32,
    ) -> Result<Capability, Fault> {
        let address = first.address().wrapping_add(immediate);
        let authority_permissions =
            self.check_memory_access(authority, address, 8, Access::Load)?;
        let mut loaded =
            self.bus.read_capability(address).map_err(|_| Fault {
                cause: 5,
                value: address,
            })?;
        #[cfg(feature = "verification")]
        self.trace.push(MemoryAccess::Read {
            address,
            width: 8,
            value: loaded.to_memory().0,
            tag: Some(loaded.tag()),
        });

        if authority_permissions & permissions::LOAD_STORE_CAPABILITY == 0 {
            loaded = loaded.clear_tag();
        }

        if loaded.tag() {
            let mut mask = 0xfff;
            let sealed = loaded.sealed();

            if authority_permissions & permissions::LOAD_GLOBAL == 0 {
                mask &= !permissions::GLOBAL;

                if !sealed {
                    mask &= !permissions::LOAD_GLOBAL;
                }
            }

            if !sealed && authority_permissions & permissions::LOAD_MUTABLE == 0
            {
                mask &= !(permissions::STORE | permissions::LOAD_MUTABLE);
            }

            loaded = loaded.restrict_permissions(mask);

            if let Some((offset, bit)) =
                loaded.revocation_bit(self.processor.revocation_memory_base)
            {
                let bitmap_address =
                    self.processor.revocation_base.checked_add(offset).ok_or(
                        Fault {
                            cause: 5,
                            value: address,
                        },
                    )?;
                let byte = self.bus.inspect(bitmap_address.into(), 1).map_err(
                    |_| Fault {
                        cause: 5,
                        value: address,
                    },
                )?;
                #[cfg(feature = "verification")]
                self.trace.push(MemoryAccess::RevocationRead {
                    address: bitmap_address,
                    value: byte as u8,
                });
                loaded = loaded.filter_revoked(byte as u8 & bit != 0);
            }
        }

        Ok(loaded)
    }

    pub(super) fn special(
        &mut self,
        index: u8,
        source: u8,
        mut value: Capability,
    ) -> Result<Capability, Fault> {
        if !(28..=31).contains(&index) {
            return Err(Fault::illegal());
        }
        self.require_system_registers(32 | index)?;
        let old = match index {
            28 => self.processor.machine_trap,
            29 => self.processor.machine_data,
            30 => self.processor.machine_scratch,
            _ => self.processor.machine_exception,
        };
        if source != 0 {
            if index == 28 || index == 31 {
                let mask = if index == 28 { 3 } else { 1 };
                let invalid = value.address() & mask != 0
                    || value.sealed()
                    || value.permissions() & permissions::EXECUTE == 0;
                let bits = value.to_memory().0 & !u64::from(mask);
                value = Capability::from_memory(bits, value.tag() && !invalid);
            }
            match index {
                28 => self.processor.machine_trap = value,
                29 => self.processor.machine_data = value,
                30 => self.processor.machine_scratch = value,
                _ => self.processor.machine_exception = value,
            }
        }

        Ok(old)
    }

    fn store_capability(
        &mut self,
        authority: u8,
        first: Capability,
        second: Capability,
        immediate: u32,
    ) -> Result<(), Fault> {
        let address = first.address().wrapping_add(immediate);
        let access = if second.tag() {
            Access::StoreCapability
        } else {
            Access::Store
        };
        let authority_permissions =
            self.check_memory_access(authority, address, 8, access)?;
        let stored = if authority_permissions & permissions::STORE_LOCAL == 0
            && second.permissions() & permissions::GLOBAL == 0
        {
            second.clear_tag()
        } else {
            second
        };
        self.bus
            .write_capability(address, stored)
            .map_err(|_| Fault {
                cause: 7,
                value: address,
            })?;
        self.record_store(address, 8);
        #[cfg(feature = "verification")]
        self.trace.push(MemoryAccess::Write {
            address,
            width: 8,
            value: stored.to_memory().0,
            tag: Some(stored.tag()),
        });
        Ok(())
    }
}
