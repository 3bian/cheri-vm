// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::Plain;
use super::Model;
use crate::hart::Hart;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::Instruction;
use crate::isa::pointer::{Access, Integer};
use crate::isa::xlen::Xlen;
use crate::machine::error::{EmulatorError, ExceptionCause};

pub enum Flat {}

impl<X: Xlen> Model<X> for Flat {
    const CAPABILITIES: bool = false;
    type Pointer = Integer<X::Word>;
    type Shadow = ();

    #[inline]
    fn authority((): &(), program_counter: X::Word) -> X::Word {
        program_counter
    }

    #[inline]
    fn call(
        (): &mut (),
        _: u8,
        _: Option<(u8, X::Word, i32)>,
        _: X::Word,
        _: X::Word,
    ) {
    }

    #[inline]
    fn check(
        (): &(),
        _: u8,
        _: X::Word,
        _: usize,
        _: Access,
    ) -> Result<(), ExceptionCause> {
        Ok(())
    }

    #[inline]
    fn clear((): &mut (), _: u8) {}

    #[inline]
    fn control<P: PrivilegeModel>(
        _: &mut Hart<Plain<X, P, Self>>,
        _: Instruction,
    ) -> Option<Result<(), EmulatorError>> {
        None
    }

    fn execute<P: PrivilegeModel>(
        _: &mut Hart<Plain<X, P, Self>>,
        _: Instruction,
        _: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        Err(EmulatorError::Refused(
            "this machine has no capabilities".into(),
        ))
    }

    #[inline]
    fn jump((): &mut (), _: X::Word, _: X::Word) {}

    #[inline]
    fn permits_system((): &()) -> bool {
        true
    }

    #[inline]
    fn returned((): &mut (), _: bool, _: X::Word) {}

    fn shadow(_: ExtensionSet) {}

    #[inline]
    fn trap((): &mut (), _: bool, _: X::Word, _: X::Word) {}

    #[inline]
    fn upper((): &mut (), _: u8, _: X::Word, _: X::Word) {}
}
