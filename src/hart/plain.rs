// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod debug;
mod execute;
pub mod model;
mod present;
mod processor;

pub use model::{Flat, Model};
pub use processor::Processor;

use core::marker::PhantomData;

use super::fault::INTERRUPT;
use super::privilege::{MachineOnly, Mode, PrivilegeModel, Reach, Supervisor};
use super::{Architecture, Exception, Fault, Hart};
use crate::isa::decode::DecodedInstruction;
use crate::isa::extension::ExtensionSet;
use crate::isa::pointer::{Rvy, Violation};
use crate::isa::xlen::{Rv32, Rv64, Word, Xlen};
use crate::machine::bus::Bus;
use crate::machine::error::ExceptionCause;

pub type Cva6 = Plain<Rv64, Supervisor>;

pub type Cva6Cheri = Plain<Rv64, Supervisor, Rvy>;

pub struct Plain<X, P = MachineOnly, M = Flat>(PhantomData<(X, P, M)>);

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Architecture for Plain<X, P, M> {
    type Pointer = M::Pointer;
    type Processor = Processor<X, P, M>;
    type Trace = ();
    type Xlen = X;

    fn enter_trap(
        processor: &mut Processor<X, P, M>,
        address: X::Word,
        fault: Fault<X::Word>,
    ) {
        processor.reservation = None;
        let handler = processor.privilege.enter_trap(
            &mut processor.machine_registers,
            address,
            fault.cause & !INTERRUPT,
            fault.value,
            fault.cause & INTERRUPT != 0,
        );
        if M::CAPABILITIES {
            let supervisor = processor.privilege.mode() == Mode::Supervisor;
            let vector = processor
                .privilege
                .read_csr(
                    &processor.machine_registers,
                    if supervisor { 0x105 } else { 0x305 },
                )
                .unwrap_or(handler);
            M::trap(&mut processor.shadow, supervisor, vector, handler);
        }
        processor.program_counter = handler;
    }

    fn execute(
        hart: &mut Hart<Self>,
        decoded: DecodedInstruction,
    ) -> Result<(), Exception<X::Word>> {
        hart.execute(decoded)
    }

    fn extensions(processor: &Processor<X, P, M>) -> ExtensionSet {
        processor.extensions
    }

    #[inline]
    fn fetch_address(
        processor: &mut Processor<X, P, M>,
        bus: &mut Bus,
        address: X::Word,
    ) -> Result<u64, Fault<X::Word>> {
        processor
            .privilege
            .translate(
                &processor.machine_registers,
                bus,
                address,
                2,
                Reach::Fetch,
            )
            .map_err(|cause| Fault {
                cause: cause.code(),
                value: address,
            })
    }

    fn fetch_fault(
        processor: &mut Processor<X, P, M>,
        _: Violation,
    ) -> Fault<X::Word> {
        Fault {
            cause: if M::CAPABILITIES {
                ExceptionCause::CheriInstructionAccessFault
            } else {
                ExceptionCause::InstructionAccessFault
            }
            .code(),
            value: processor.program_counter,
        }
    }

    fn illegal_instruction(encoded: u32) -> Fault<X::Word> {
        Fault {
            cause: ExceptionCause::IllegalInstruction.code(),
            value: X::Word::from_u64(u64::from(encoded)),
        }
    }

    #[inline]
    fn pending_interrupt(
        processor: &mut Processor<X, P, M>,
        lines: u32,
    ) -> Option<u32> {
        processor
            .privilege
            .interrupt(&mut processor.machine_registers, lines)
    }
}

pub type Rv32Plain = Plain<Rv32>;

pub type Rv64Plain = Plain<Rv64>;

pub type Rv64Rvy = Plain<Rv64, MachineOnly, Rvy>;
