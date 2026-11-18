// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::debug::architecture::DebugArchitecture;
use gdbstub::arch::Arch;

pub(crate) trait GdbArchitecture: DebugArchitecture {
    const ANY_BREAKPOINT_ADDRESS: bool;
    type Arch: Arch<BreakpointKind = usize, Usize: Into<u64>>;
    type Context: Copy;
    const MONITOR_HELP: &'static str;

    fn context(processor: &Self::Processor) -> Self::Context;

    fn describe_context(context: &Self::Context) -> String;

    fn describe_processor(processor: &Self::Processor) -> String;

    fn read_register(
        processor: &Self::Processor,
        id: <Self::Arch as Arch>::RegId,
        bytes: &mut [u8],
    ) -> Option<usize>;

    fn registers(
        processor: &Self::Processor,
    ) -> <Self::Arch as Arch>::Registers;

    fn target_description(_processor: &Self::Processor) -> Option<String> {
        None
    }
}
