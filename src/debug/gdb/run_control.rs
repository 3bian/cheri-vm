// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use gdbstub::common::Signal;

use super::{BreakpointType, RunState, StopReason};
use crate::machine::EmulatorError;

pub(crate) struct RunControl {
    breakpoints: BTreeSet<(u64, BreakpointType, usize)>,
    bypass: Option<u64>,
    maximum_steps: Option<u64>,
}

impl RunControl {
    pub(crate) fn before_step(
        &mut self,
        address: u64,
        attempted_steps: u64,
    ) -> Option<StopReason> {
        let bypass = self.bypass.take() == Some(address);
        if !bypass {
            for kind in [BreakpointType::Software, BreakpointType::Hardware] {
                if self.breakpoints.iter().any(|(location, registered, _)| {
                    *location == address && *registered == kind
                }) {
                    return Some(StopReason::Breakpoint(address, kind));
                }
            }
        }
        self.maximum_steps
            .filter(|limit| attempted_steps >= *limit)
            .map(|_| StopReason::RunLimit)
    }

    pub(crate) fn insert_breakpoint(
        &mut self,
        accepts: impl FnOnce(u64, usize) -> bool,
        breakpoint: BreakpointType,
        address: u64,
        kind: usize,
        alignment: u64,
    ) -> bool {
        if !address.is_multiple_of(alignment)
            || !(kind == 4 || (kind == 2 && alignment == 2))
            || address.checked_add((kind - 1) as u64).is_none()
            || !accepts(address, kind)
            || (self.breakpoints.len() == 64
                && !self.breakpoints.contains(&(address, breakpoint, kind)))
        {
            return false;
        }
        self.breakpoints.insert((address, breakpoint, kind));
        true
    }

    pub(crate) fn new(maximum_steps: Option<u64>) -> Self {
        Self {
            breakpoints: BTreeSet::new(),
            bypass: None,
            maximum_steps: maximum_steps.filter(|limit| *limit != 0),
        }
    }

    pub(crate) fn remove_breakpoint(
        &mut self,
        breakpoint: BreakpointType,
        address: u64,
        kind: usize,
        alignment: u64,
    ) -> bool {
        if !address.is_multiple_of(alignment)
            || !(kind == 4 || (kind == 2 && alignment == 2))
        {
            return false;
        }
        self.breakpoints.remove(&(address, breakpoint, kind));
        true
    }

    pub(crate) fn resume(
        &mut self,
        current: &mut RunState,
        reason: StopReason,
        next: RunState,
        signal: Option<Signal>,
    ) -> Result<(), EmulatorError> {
        if *current != RunState::Halted {
            return Err(EmulatorError::Execution(
                "debugger resume requires a halted target".into(),
            ));
        }
        if signal.is_some_and(|signal| signal != Signal::SIGZERO) {
            return Err(EmulatorError::Execution(
                "debugger signal injection is unsupported".into(),
            ));
        }
        if let StopReason::Breakpoint(address, _) = reason {
            self.bypass = Some(address);
        }
        *current = next;
        Ok(())
    }
}
