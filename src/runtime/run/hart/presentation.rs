// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::execution::StopReason;
use crate::hart::{Architecture, Hart, StepOutcome, WordOf};
use crate::isa::decode::DecodedInstruction;
use crate::machine::EmulatorError;
use crate::runtime::options::run::RunOptions;
use crate::runtime::output::{GuestOutput, OutputPresentation};
use std::io::Write;

pub(crate) trait Presentation: Architecture {
    const FLUSH_WHEN_TRACING: bool;
    const REPORT_TRAPS: bool;
    const STOPS_IN_STEP: bool;

    fn handle_error(
        error: EmulatorError,
        _output: &mut GuestOutput<impl Write>,
        _presentation: OutputPresentation,
    ) -> Result<StopReason, EmulatorError> {
        Err(error)
    }

    fn handle_step(
        hart: &mut Hart<Self>,
        outcome: &StepOutcome<WordOf<Self>>,
        options: &RunOptions,
        scratch: &mut Vec<u8>,
        output: &mut GuestOutput<impl Write>,
        traps: &mut u64,
    ) -> Result<Option<StopReason>, EmulatorError>;

    fn is_wait(decoded: &DecodedInstruction) -> bool;

    fn present_state(
        hart: &Hart<Self>,
        options: &RunOptions,
        output: &mut GuestOutput<impl Write>,
    ) -> Result<(), EmulatorError>;
}
