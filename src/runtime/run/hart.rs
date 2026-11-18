// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod presentation;
mod runnable;

pub(crate) use presentation::Presentation;
pub(crate) use runnable::Runnable;

use std::io::Write;

use crate::hart::execution::{ExecutionReport, StepReport, StopReason};

use crate::hart::{Hart, ProcessorState, StepOutcome, WordOf};

use crate::isa::xlen::Word;

use crate::machine::EmulatorError;

use crate::machine::TimingPolicy;

use crate::machine::bus::Bus;

use crate::machine::completion::HostInterface;

use super::run_target::RunTarget;

use crate::runtime::options::run::RunOptions;

use crate::runtime::output::{GuestOutput, OutputPresentation};

impl<A: Presentation> RunTarget for Hart<A> {
    const FLUSH_WHEN_TRACING: bool = A::FLUSH_WHEN_TRACING;
    type Outcome = StepOutcome<WordOf<A>>;
    const REPORT_TRAPS: bool = A::REPORT_TRAPS;

    fn execution_report(&self, stop: StopReason) -> ExecutionReport {
        Hart::execution_report(self, stop)
    }

    fn handle_error(
        &mut self,
        error: EmulatorError,
        output: &mut GuestOutput<impl Write>,
        presentation: OutputPresentation,
    ) -> Result<StopReason, EmulatorError> {
        A::handle_error(error, output, presentation)
    }

    fn handle_step(
        &mut self,
        outcome: &Self::Outcome,
        options: &RunOptions,
        scratch: &mut Vec<u8>,
        output: &mut GuestOutput<impl Write>,
        traps: &mut u64,
    ) -> Result<Option<StopReason>, EmulatorError> {
        A::handle_step(self, outcome, options, scratch, output, traps)
    }

    fn io_parts(&mut self) -> (&mut Bus, &mut Option<HostInterface>) {
        (&mut self.bus, &mut self.host_interface)
    }

    fn is_waiting(&self, outcome: &Self::Outcome) -> bool {
        let waits = matches!(
            outcome,
            StepOutcome::Retired { decoded, .. } if A::is_wait(decoded)
        );
        self.timing == TimingPolicy::Elapsed && waits
    }

    fn position(&self) -> (u64, u64) {
        (
            self.processor.program_counter().to_u64(),
            self.attempted_steps,
        )
    }

    fn present_state(
        &self,
        options: &RunOptions,
        output: &mut GuestOutput<impl Write>,
    ) -> Result<(), EmulatorError> {
        A::present_state(self, options, output)
    }

    fn stack_pointer(&self) -> u64 {
        self.processor.stack_pointer().to_u64()
    }

    fn step_report(
        &mut self,
    ) -> Result<StepReport<Self::Outcome>, EmulatorError> {
        Hart::step_report(self)
    }
}

pub(crate) fn run_with_options<A: Presentation>(
    mut hart: Hart<A>,
    options: RunOptions,
) -> Result<(Hart<A>, ExecutionReport), EmulatorError> {
    if A::STOPS_IN_STEP {
        hart.trap_policy = options.trap_policy;
    }
    super::run_with_options(hart, options)
}
