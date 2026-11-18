// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod flat;

pub use flat::Flat;

pub mod rvy;

use super::Plain;

use crate::hart::Hart;

use crate::hart::privilege::PrivilegeModel;

use crate::isa::extension::ExtensionSet;

use crate::isa::instruction::Instruction;

use crate::isa::pointer::{Access, PointerModel};

use crate::isa::xlen::Xlen;

use crate::machine::error::{EmulatorError, ExceptionCause};

pub type FetchOf<X, M> = <<M as Model<X>>::Pointer as PointerModel>::Fetch;

pub trait Model<X: Xlen>: Sized + 'static {
    const CAPABILITIES: bool;
    type Pointer: PointerModel<Address = X::Word>;
    type Shadow;

    fn authority(
        shadow: &Self::Shadow,
        program_counter: X::Word,
    ) -> FetchOf<X, Self>;

    fn call(
        shadow: &mut Self::Shadow,
        destination: u8,
        base: Option<(u8, X::Word, i32)>,
        next: X::Word,
        target: X::Word,
    );

    fn check(
        shadow: &Self::Shadow,
        base: u8,
        target: X::Word,
        width: usize,
        access: Access,
    ) -> Result<(), ExceptionCause>;

    fn clear(shadow: &mut Self::Shadow, register: u8);

    fn control<P: PrivilegeModel>(
        hart: &mut Hart<Plain<X, P, Self>>,
        instruction: Instruction,
    ) -> Option<Result<(), EmulatorError>>;

    fn execute<P: PrivilegeModel>(
        hart: &mut Hart<Plain<X, P, Self>>,
        instruction: Instruction,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError>;

    fn jump(shadow: &mut Self::Shadow, from: X::Word, target: X::Word);

    fn permits_system(shadow: &Self::Shadow) -> bool;

    fn returned(shadow: &mut Self::Shadow, supervisor: bool, target: X::Word);

    fn shadow(extensions: ExtensionSet) -> Self::Shadow;

    fn trap(
        shadow: &mut Self::Shadow,
        supervisor: bool,
        vector: X::Word,
        handler: X::Word,
    );

    fn upper(
        shadow: &mut Self::Shadow,
        destination: u8,
        from: X::Word,
        value: X::Word,
    );
}

pub type RegisterOf<X, M> =
    <<M as Model<X>>::Pointer as PointerModel>::Register;
