// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::hart::execution::{ExecutionReport, StepReport, StopReason};
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::completion::HostInterface;

use crate::runtime::options::run::RunOptions;
use crate::runtime::output::{GuestOutput, OutputPresentation};

pub(crate) trait RunTarget {
    const FLUSH_WHEN_TRACING: bool = false;
    type Outcome;
    const REPORT_TRAPS: bool = false;

    fn execution_report(&self, stop: StopReason) -> ExecutionReport;

    fn handle_error(
        &mut self,
        error: EmulatorError,
        _output: &mut GuestOutput<impl Write>,
        _presentation: OutputPresentation,
    ) -> Result<StopReason, EmulatorError> {
        Err(error)
    }

    fn handle_step(
        &mut self,
        outcome: &Self::Outcome,
        options: &RunOptions,
        scratch: &mut Vec<u8>,
        output: &mut GuestOutput<impl Write>,
        traps: &mut u64,
    ) -> Result<Option<StopReason>, EmulatorError>;

    fn io_parts(&mut self) -> (&mut Bus, &mut Option<HostInterface>);

    fn is_waiting(&self, outcome: &Self::Outcome) -> bool;

    fn position(&self) -> (u64, u64);

    fn present_state(
        &self,
        options: &RunOptions,
        output: &mut GuestOutput<impl Write>,
    ) -> Result<(), EmulatorError>;

    fn stack_pointer(&self) -> u64;

    fn step_report(
        &mut self,
    ) -> Result<StepReport<Self::Outcome>, EmulatorError>;
}
