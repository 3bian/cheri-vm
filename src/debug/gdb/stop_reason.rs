// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::BreakpointType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StopReason {
    Breakpoint(u64, BreakpointType),
    Completed(u64),
    Ebreak(u64),
    Fault(u32),
    Interrupted,
    OutputLimit,
    RunLimit,
    Startup,
    Step,
}

impl StopReason {
    pub(crate) fn from_fault(
        address: u64,
        cause: u32,
        interrupt: bool,
    ) -> Option<Self> {
        if interrupt {
            None
        } else if cause == 3 {
            Some(Self::Ebreak(address))
        } else {
            Some(Self::Fault(cause))
        }
    }
}
