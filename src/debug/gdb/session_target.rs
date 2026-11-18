// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use gdbstub::target::Target;

use super::StopReason;
use crate::machine::EmulatorError;

pub(crate) trait SessionTarget: Target<Error = EmulatorError> {
    fn advance(&mut self) -> Result<Option<StopReason>, EmulatorError>;

    fn drain_output_each(
        &mut self,
        scratch: &mut Vec<u8>,
        each: &mut impl FnMut(&str, &str, &[u8]) -> Result<(), EmulatorError>,
    ) -> Result<(), EmulatorError>;

    fn halt(&mut self, reason: StopReason) -> StopReason;

    fn poll_input(
        &mut self,
        _input: &mut crate::machine::host::uart::input::UartInput,
    ) -> Result<(), EmulatorError> {
        Err(EmulatorError::Configuration(
            "this debugger target does not support live uart input".into(),
        ))
    }
}
