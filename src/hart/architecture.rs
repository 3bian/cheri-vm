// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod processor_state;

pub use processor_state::ProcessorState;

use super::{Exception, Fault, Hart};

use crate::isa::decode::DecodedInstruction;

use crate::isa::extension::ExtensionSet;

use crate::isa::pointer::{PointerModel, Violation};

use crate::isa::xlen::{Word, Xlen};

use crate::machine::bus::Bus;

pub trait Architecture: Sized {
    type Pointer: PointerModel<Address = <Self::Xlen as Xlen>::Word>;
    type Processor: ProcessorState<Self::Pointer>;
    type Trace: Default;
    type Xlen: Xlen;

    fn begin_step(_hart: &mut Hart<Self>) {}

    fn end_step(_hart: &mut Hart<Self>) {}

    fn enter_trap(
        processor: &mut Self::Processor,
        address: WordOf<Self>,
        fault: Fault<WordOf<Self>>,
    );

    fn execute(
        hart: &mut Hart<Self>,
        decoded: DecodedInstruction,
    ) -> Result<(), Exception<WordOf<Self>>>;

    fn extensions(processor: &Self::Processor) -> ExtensionSet;

    #[inline]
    fn fetch_address(
        _processor: &mut Self::Processor,
        _bus: &mut Bus,
        address: WordOf<Self>,
    ) -> Result<u64, Fault<WordOf<Self>>> {
        Ok(address.to_u64())
    }

    fn fetch_fault(
        processor: &mut Self::Processor,
        violation: Violation,
    ) -> Fault<WordOf<Self>>;

    fn illegal_instruction(encoded: u32) -> Fault<WordOf<Self>>;

    fn pending_interrupt(
        processor: &mut Self::Processor,
        lines: u32,
    ) -> Option<u32>;
}

pub type WordOf<A> = <<A as Architecture>::Xlen as Xlen>::Word;
