// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod capability_fault_cause;
mod capability_memory;
pub(crate) mod debug;
mod execute;
#[cfg(feature = "verification")]
mod memory_access;
mod present;
mod processor;
#[cfg(feature = "verification")]
mod register_access;

pub(crate) use capability_fault_cause::CapabilityFaultCause;
pub(crate) use capability_memory::CapabilityMemory;
#[cfg(feature = "verification")]
pub use memory_access::MemoryAccess;
pub use processor::Processor;
#[cfg(feature = "verification")]
pub(crate) use register_access::RegisterAccess;

use super::fault::INTERRUPT;
use super::privilege::highest_interrupt;
use super::{Architecture, Exception, Fault, Hart};
use crate::isa::decode::DecodedInstruction;
use crate::isa::extension::ExtensionSet;
use crate::isa::pointer::{Cheriot, Violation};
use crate::isa::xlen::Rv32;

const SOFTWARE_INTERRUPTS: u32 = 0x222;

pub enum CheriotRv32e {}

impl Architecture for CheriotRv32e {
    type Pointer = Cheriot;
    type Processor = Processor;
    #[cfg(not(feature = "verification"))]
    type Trace = ();
    #[cfg(feature = "verification")]
    type Trace = Vec<MemoryAccess>;
    type Xlen = Rv32;

    #[cfg(feature = "verification")]
    fn begin_step(hart: &mut Hart<Self>) {
        hart.processor.begin_observation();
        hart.trace.clear();
    }

    fn end_step(hart: &mut Hart<Self>) {
        let lines = hart.bus.machine_interrupt_bits();
        let registers = &mut hart.processor.machine_registers;
        registers.interrupt_pending =
            (registers.interrupt_pending & SOFTWARE_INTERRUPTS) | lines;
        #[cfg(feature = "verification")]
        hart.processor.end_observation();
    }

    fn enter_trap(processor: &mut Processor, address: u32, fault: Fault) {
        processor.machine_exception = processor.pcc.set_address(address);
        processor.machine_registers.enter_trap(
            address,
            fault.cause & !INTERRUPT,
            fault.value,
            fault.cause & INTERRUPT != 0,
        );
        processor.pcc = processor.machine_trap;
        processor.program_counter = processor.machine_trap.address();
    }

    fn execute(
        hart: &mut Hart<Self>,
        decoded: DecodedInstruction,
    ) -> Result<(), Exception> {
        hart.execute(decoded).map_err(Exception::Fault)
    }

    fn extensions(_: &Processor) -> ExtensionSet {
        Processor::EXTENSIONS
    }

    fn fetch_fault(processor: &mut Processor, violation: Violation) -> Fault {
        if violation == Violation::Bounds {
            processor.pcc = processor.pcc.clear_tag();
        }
        Fault::capability_cause(
            match violation {
                Violation::Tag => CapabilityFaultCause::Tag,
                Violation::Seal => CapabilityFaultCause::Seal,
                Violation::Bounds => CapabilityFaultCause::Bounds,
                _ => CapabilityFaultCause::ExecutePermission,
            },
            Processor::PCC,
        )
    }

    fn illegal_instruction(_: u32) -> Fault {
        Fault::illegal()
    }

    #[inline]
    fn pending_interrupt(processor: &mut Processor, lines: u32) -> Option<u32> {
        let registers = &mut processor.machine_registers;
        registers.interrupt_pending =
            (registers.interrupt_pending & SOFTWARE_INTERRUPTS) | lines;
        let enabled = registers.interrupt_pending & registers.interrupt_enable;
        if registers.machine_status & 8 == 0 || enabled == 0 {
            return None;
        }
        Some(highest_interrupt(enabled))
    }
}
