// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![cfg(feature = "verification")]

use crate::hart::cheriot_rv32e::{CheriotRv32e, MemoryAccess};
use crate::hart::{Hart, StepOutcome};

impl Hart<CheriotRv32e> {
    pub(crate) fn memory_accesses(&self) -> &[MemoryAccess] {
        &self.trace
    }

    pub(crate) fn step_injected(&mut self, encoded: u32) -> StepOutcome {
        match self.step_with(Some(encoded)) {
            Ok(outcome) => outcome,
            Err(error) => panic!("an injected step failed: {error}"),
        }
    }
}
