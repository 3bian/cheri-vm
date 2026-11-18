// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::TimingPolicy;

pub(crate) struct StepClock {
    period: u32,
    phase: u32,
    started: bool,
}

impl StepClock {
    pub(crate) fn advance(&mut self) -> bool {
        self.started = true;
        self.phase += 1;
        if self.phase == self.period {
            self.phase = 0;
            true
        } else {
            false
        }
    }

    pub(crate) fn set_period(
        &mut self,
        timing: TimingPolicy,
        period: u32,
    ) -> Result<(), EmulatorError> {
        super::validate_step_period(timing, period)?;
        if self.started {
            return Err(EmulatorError::Configuration(
                "step quantum cannot change after execution starts".into(),
            ));
        }
        self.period = period;
        Ok(())
    }
}

impl Default for StepClock {
    fn default() -> Self {
        Self {
            period: super::DEFAULT_STEPS_PER_TICK,
            phase: 0,
            started: false,
        }
    }
}
