// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::machine_registers::MachineRegisters;
use crate::isa::pointer::PointerModel;

pub trait ProcessorState<P: PointerModel> {
    fn fetch_authority(&self) -> P::Fetch;

    fn machine_registers(&self) -> &MachineRegisters<P::Address>;

    fn machine_registers_mut(&mut self) -> &mut MachineRegisters<P::Address>;

    fn program_counter(&self) -> P::Address;

    fn stack_pointer(&self) -> P::Address;
}
