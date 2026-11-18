// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::extension::ExtensionSet;
use crate::isa::extension::rvy::Operation;
use crate::isa::instruction::Instruction;
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::Capability128;
use crate::isa::xlen::Rv64;
use crate::machine::error::EmulatorError;

impl<P: PrivilegeModel> Hart<Plain<Rv64, P, Rvy>> {
    pub(super) fn capability(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), EmulatorError> {
        use Operation::*;
        let Instruction::Rvy {
            operation,
            destination,
            first,
            second,
            immediate,
        } = instruction
        else {
            return Err(EmulatorError::Execution(
                "not a capability instruction".to_owned(),
            ));
        };
        let processor = &mut self.processor;
        let source = processor.capability(first);
        let other = processor.capability(second);
        let value = other.address;
        let integer = Capability128::integer;
        let (base, top) = source.bounds();
        let result = match operation {
            Add => source.with_address(source.address.wrapping_add(value)),
            AddImmediate => source.with_address(
                source.address.wrapping_add(immediate as i64 as u64),
            ),
            AlignmentMask => {
                integer(Capability128::alignment_mask(source.address))
            }
            Base => integer(base),
            Build => Capability128 {
                tag: source.tag
                    && !source.sealed()
                    && other.is_subset_of(source),
                ..other
            },
            ClearPermissions => source.without_permissions(value),
            Equal => integer((source == other).into()),
            High => integer(source.metadata),
            Length => integer(
                u64::try_from(top.saturating_sub(base.into()))
                    .unwrap_or(u64::MAX),
            ),
            Move => source,
            Permissions => integer(processor.shadow.seen_permissions(source)),
            SealEntry => Capability128 {
                tag: source.tag && !source.sealed() && source.is_sound(),
                ..source.with_seal(true)
            },
            SetAddress => source.with_address(value),
            SetBounds => source.with_bounds(value, true),
            SetBoundsImmediate => {
                source.with_bounds(immediate as u16 as u64, true)
            }
            SetBoundsRounded => source.with_bounds(value, false),
            SetHigh => Capability128 {
                address: source.address,
                metadata: value,
                tag: false,
            },
            ShiftAdd | ShiftAddUnsignedWord => {
                if !processor.extensions.contains(ExtensionSet::ZBA) {
                    return Err(EmulatorError::Refused(
                        "the capability shift-and-add instructions need zba"
                            .into(),
                    ));
                }
                let index = if operation == ShiftAdd {
                    source.address
                } else {
                    source.address & 0xffff_ffff
                };
                other.with_address(
                    other.address.wrapping_add(index << immediate),
                )
            }
            Subset => integer(
                (source.tag == other.tag && other.is_subset_of(source)).into(),
            ),
            Tag => integer(source.tag.into()),
            Top => integer(u64::try_from(top).unwrap_or(u64::MAX)),
            Type => integer(source.sealed().into()),
            Unseal => Capability128 {
                tag: source.tag
                    && !source.sealed()
                    && other.tag
                    && other.sealed()
                    && other.is_subset_of(source),
                ..other.with_seal(false)
            },
        };
        processor.set_capability(destination, result);
        Ok(())
    }
}
