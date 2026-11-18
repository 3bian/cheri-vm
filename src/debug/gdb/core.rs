// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod debug_fault;
mod gdb_architecture;
mod stepped;
mod trap;

pub(crate) use debug_fault::DebugFault;
pub(crate) use gdb_architecture::GdbArchitecture;
pub(crate) use stepped::Stepped;
pub(crate) use trap::Trap;

use gdbstub::arch::Arch;

use crate::machine::EmulatorError;

use crate::machine::host::uart::input::UartInput;

pub(crate) trait Core {
    type Arch: Arch<BreakpointKind = usize, Usize: Into<u64>>;
    type Fault: Copy;

    const MONITOR_HELP: &'static str;

    fn accepts_breakpoint(&self, address: u64, kind: usize) -> bool;

    fn attempted_steps(&self) -> u64;

    fn describe_fault(fault: &Self::Fault) -> String;

    fn drain_output_each(
        &mut self,
        scratch: &mut Vec<u8>,
        each: &mut impl FnMut(&str, &str, &[u8]) -> Result<(), EmulatorError>,
    ) -> Result<(), EmulatorError>;

    fn inspect(
        &self,
        address: u64,
        data: &mut [u8],
    ) -> Result<(), EmulatorError>;

    fn minimum_alignment(&self) -> u64;

    fn monitor(&self, words: &[&str]) -> Option<Result<String, &'static str>> {
        let _ = words;
        None
    }

    fn pause_host_clock(&mut self);

    fn poll_input(
        &mut self,
        input: &mut UartInput,
    ) -> Result<(), EmulatorError>;

    fn program_counter(&self) -> u64;

    fn read_register(
        &self,
        id: <Self::Arch as Arch>::RegId,
        bytes: &mut [u8],
    ) -> Option<usize>;

    fn registers(&self) -> <Self::Arch as Arch>::Registers;

    fn resume_host_clock(&mut self);

    fn status(&self) -> String;

    fn step(&mut self) -> Result<Stepped<Self::Fault>, EmulatorError>;

    fn target_description(&self) -> Option<String> {
        None
    }
}
