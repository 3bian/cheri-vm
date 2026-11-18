// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::{PrivilegeModel, Reach};
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{AtomicOperation, Instruction};
use crate::isa::pointer::rvy::{Capability128, permissions};
use crate::isa::pointer::{Access, Rvy};
use crate::isa::xlen::Rv64;
use crate::machine::error::{EmulatorError, ExceptionCause};

impl<P: PrivilegeModel> Hart<Plain<Rv64, P, Rvy>> {
    fn authorize(
        &self,
        base: u8,
        target: u64,
        access: Access,
    ) -> Result<(), EmulatorError> {
        let trap = |cause| EmulatorError::Trap {
            cause,
            address: self.processor.program_counter,
            value: target,
        };
        Rvy::check(&self.processor.shadow, base, target, 16, access)
            .map_err(trap)?;
        if !target.is_multiple_of(16) {
            return Err(trap(if access.writes() {
                ExceptionCause::StoreAccessFault
            } else {
                ExceptionCause::LoadAccessFault
            }));
        }
        Ok(())
    }

    pub(super) fn capability_memory(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), EmulatorError> {
        match instruction {
            Instruction::Load {
                destination,
                base,
                offset,
                ..
            } => {
                let target = self.target(base, offset);
                let loaded = self.load_capability(base, target, Reach::Load)?;
                self.processor.set_capability(destination, loaded);
            }
            Instruction::Store {
                source,
                base,
                offset,
                ..
            } => {
                let target = self.target(base, offset);
                self.authorize(base, target, Access::Store)?;
                self.store_capability(
                    base,
                    target,
                    self.processor.capability(source),
                )?;
            }
            Instruction::Atomic {
                operation,
                destination,
                address: base,
                source,
                ..
            } => {
                if !self.processor.extensions.contains(ExtensionSet::A) {
                    return Err(EmulatorError::Refused(
                        "the atomic capability instructions need a".into(),
                    ));
                }
                let target = self.target(base, 0);
                let stored = self.processor.capability(source);
                let result = match operation {
                    AtomicOperation::LoadReserved => {
                        let loaded =
                            self.load_capability(base, target, Reach::Load)?;
                        self.processor.reservation = Some(target);
                        loaded
                    }
                    AtomicOperation::StoreConditional => {
                        self.authorize(base, target, Access::Store)?;
                        let held =
                            self.processor.reservation.take() == Some(target);
                        if held {
                            self.store_capability(base, target, stored)?;
                        }
                        Capability128::integer((!held).into())
                    }
                    _ => {
                        self.authorize(base, target, Access::Store)?;
                        let tagged = self.stored_tag(base, target, stored)?;
                        let loaded =
                            self.load_capability(base, target, Reach::Store)?;
                        self.write_capability(target, stored, tagged)?;
                        loaded
                    }
                };
                self.processor.set_capability(destination, result);
            }
            _ => {
                return Err(EmulatorError::Execution(
                    "not a capability access".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn load_capability(
        &mut self,
        base: u8,
        target: u64,
        reach: Reach,
    ) -> Result<Capability128, EmulatorError> {
        self.authorize(base, target, Access::Load)?;
        let address = self.load(target, 8, reach)?;
        let metadata = self.load(target.wrapping_add(8), 8, reach)?;
        let held = self.processor.capability(base).permissions();
        let tagged =
            self.memory_tag(target)? && held & permissions::CAPABILITY != 0;
        let mut loaded = Capability128 {
            address,
            metadata,
            tag: tagged && self.capability_load(target, reach, tagged)?,
        };
        if !loaded.tag {
            return Ok(loaded);
        }
        if held & permissions::LOAD_MUTABLE == 0 && !loaded.sealed() {
            loaded = loaded.without_permissions(
                permissions::WRITE | permissions::LOAD_MUTABLE,
            );
        }
        if self.processor.shadow.levels && held & permissions::LOAD_GLOBAL == 0
        {
            loaded = loaded.without_permissions(if loaded.sealed() {
                permissions::GLOBAL
            } else {
                permissions::GLOBAL | permissions::LOAD_GLOBAL
            });
        }
        Ok(loaded)
    }

    fn store_capability(
        &mut self,
        base: u8,
        target: u64,
        stored: Capability128,
    ) -> Result<(), EmulatorError> {
        let tagged = self.stored_tag(base, target, stored)?;
        self.write_capability(target, stored, tagged)
    }

    fn stored_tag(
        &mut self,
        base: u8,
        target: u64,
        stored: Capability128,
    ) -> Result<bool, EmulatorError> {
        let held = self.processor.capability(base).permissions();
        let confined = self.processor.shadow.levels
            && stored.permissions() & permissions::GLOBAL == 0
            && held & permissions::STORE_LOCAL == 0;
        let tagged =
            stored.tag && held & permissions::CAPABILITY != 0 && !confined;
        if tagged {
            self.capability_store(target)?;
        }
        Ok(tagged)
    }

    fn target(&self, base: u8, offset: i32) -> u64 {
        self.processor
            .read_register_unchecked(base)
            .wrapping_add(offset as i64 as u64)
    }

    fn write_capability(
        &mut self,
        target: u64,
        stored: Capability128,
        tagged: bool,
    ) -> Result<(), EmulatorError> {
        self.store(target, 8, stored.address)?;
        self.store(target.wrapping_add(8), 8, stored.metadata)?;
        if tagged {
            self.set_memory_tag(target)?;
        }
        Ok(())
    }
}
