// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use gdbstub::arch::Arch;

use super::core::{Core, DebugFault, GdbArchitecture, Stepped, Trap};
use crate::hart::{Hart, ProcessorState, StepOutcome};
use crate::isa::xlen::{Word, Xlen};
use crate::machine::EmulatorError;
use crate::machine::host::uart::input::UartInput;

impl<A: GdbArchitecture> Core for Hart<A> {
    type Arch = A::Arch;
    type Fault = DebugFault<A::Context>;

    const MONITOR_HELP: &'static str = A::MONITOR_HELP;

    fn accepts_breakpoint(&self, address: u64, kind: usize) -> bool {
        A::ANY_BREAKPOINT_ADDRESS
            || (0..kind as u64)
                .step_by(2)
                .all(|offset| self.bus.fetch(address + offset, 2).is_ok())
    }

    fn attempted_steps(&self) -> u64 {
        self.attempted_steps
    }

    fn describe_fault(fault: &Self::Fault) -> String {
        let digits = (A::Xlen::BITS / 4) as usize;
        format!(
            "fault address=0x{:0digits$x} cause=0x{:0digits$x} \
             value=0x{:0digits$x}{} resulting_pc=0x{:0digits$x}\n",
            fault.address,
            u64::from(fault.cause)
                | u64::from(fault.interrupt) << (A::Xlen::BITS - 1),
            fault.value,
            A::describe_context(&fault.context),
            fault.resulting_pc
        )
    }

    fn drain_output_each(
        &mut self,
        scratch: &mut Vec<u8>,
        each: &mut impl FnMut(&str, &str, &[u8]) -> Result<(), EmulatorError>,
    ) -> Result<(), EmulatorError> {
        crate::runtime::host_output::drain(
            &mut self.bus,
            &mut self.host_interface,
            scratch,
            each,
        )
    }

    fn inspect(
        &self,
        address: u64,
        data: &mut [u8],
    ) -> Result<(), EmulatorError> {
        crate::debug::inspect_memory(
            &self.bus,
            |location| A::physical(&self.processor, &self.bus, location),
            address,
            data,
        )
    }

    fn minimum_alignment(&self) -> u64 {
        self.processor
            .machine_registers()
            .instruction_alignment
            .into()
    }

    fn monitor(&self, words: &[&str]) -> Option<Result<String, &'static str>> {
        A::monitor(self, words)
    }

    fn pause_host_clock(&mut self) {
        Self::pause_host_clock(self);
    }

    fn poll_input(
        &mut self,
        input: &mut UartInput,
    ) -> Result<(), EmulatorError> {
        input.poll(&mut self.bus)
    }

    fn program_counter(&self) -> u64 {
        self.processor.program_counter().to_u64()
    }

    fn read_register(
        &self,
        id: <A::Arch as Arch>::RegId,
        bytes: &mut [u8],
    ) -> Option<usize> {
        A::read_register(&self.processor, id, bytes)
    }

    fn registers(&self) -> <A::Arch as Arch>::Registers {
        A::registers(&self.processor)
    }

    fn resume_host_clock(&mut self) {
        Self::resume_host_clock(self);
    }

    fn status(&self) -> String {
        let registers = self.processor.machine_registers();
        format!(
            "steps={} retired={} virtual_time={}{}",
            self.attempted_steps,
            registers.retired,
            registers.time,
            A::describe_processor(&self.processor)
        )
    }

    fn step(&mut self) -> Result<Stepped<Self::Fault>, EmulatorError> {
        let context = A::context(&self.processor);
        let trap = match self.step_report() {
            Ok(report) => match report.outcome {
                StepOutcome::Trapped {
                    address,
                    cause,
                    interrupt,
                    value,
                } => Some((address.to_u64(), cause, interrupt, value.to_u64())),
                StepOutcome::Retired { .. } => None,
            },
            Err(EmulatorError::Trap {
                address,
                cause,
                value,
            }) => Some((address, cause.code(), false, value)),
            Err(error) => return Err(error),
        };
        Ok(Stepped {
            trap: trap.map(|(address, cause, interrupt, value)| Trap {
                address,
                cause,
                fault: DebugFault {
                    address,
                    cause,
                    context,
                    interrupt,
                    resulting_pc: self.processor.program_counter().to_u64(),
                    value,
                },
                interrupt,
            }),
            completion: self.completion,
        })
    }

    fn target_description(&self) -> Option<String> {
        A::target_description(&self.processor)
    }
}
