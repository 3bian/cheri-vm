// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod architecture;
pub mod cheriot_rv32e;
pub mod execution;
mod fault;
pub mod machine_registers;
pub mod plain;
pub mod privilege;
mod step;
mod step_outcome;
pub(crate) mod translation;

pub use architecture::{Architecture, ProcessorState, WordOf};
pub use cheriot_rv32e::CheriotRv32e;
pub use fault::{Exception, Fault};
pub use plain::{Cva6, Cva6Cheri, Rv32Plain, Rv64Plain, Rv64Rvy};
pub use step_outcome::StepOutcome;

use crate::hart::execution::{
    ExecutionReport, StepReport, StopReason, TrapPolicy,
};
use crate::isa::decode::DecodeCache;
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::completion::HostInterface;
use crate::machine::timing_policy::{HostClock, StepClock, TimingPolicy};

pub struct Hart<A: Architecture> {
    pub attempted_steps: u64,
    pub bus: Bus,
    pub completion: Option<u64>,
    decode_cache: DecodeCache,
    pub fetched_instructions: u64,
    host_clock: HostClock,
    pub host_interface: Option<HostInterface>,
    pub last_outcome: Option<StepOutcome<WordOf<A>>>,
    pub last_store: Option<(u64, usize)>,
    pub processor: A::Processor,
    step_clock: StepClock,
    pub timing: TimingPolicy,
    pub trace: A::Trace,
    pub trap_policy: TrapPolicy,
}

impl<A: Architecture> Hart<A> {
    pub fn advance(&mut self) -> Result<Option<u64>, EmulatorError> {
        self.step_report().map(|report| report.completion)
    }

    pub(crate) fn execution_report(&self, stop: StopReason) -> ExecutionReport {
        let registers = self.processor.machine_registers();
        ExecutionReport {
            attempted_steps: self.attempted_steps,
            fetched_instructions: self.fetched_instructions,
            retired_instructions: registers.retired,
            stop,
            virtual_time: registers.time,
        }
    }

    pub(crate) fn new(processor: A::Processor, bus: Bus) -> Self {
        Self {
            attempted_steps: 0,
            bus,
            completion: None,
            decode_cache: DecodeCache::new(),
            fetched_instructions: 0,
            host_clock: HostClock::default(),
            host_interface: None,
            last_outcome: None,
            last_store: None,
            processor,
            step_clock: StepClock::default(),
            timing: TimingPolicy::default(),
            trace: A::Trace::default(),
            trap_policy: TrapPolicy::Vector,
        }
    }

    pub fn pause_host_clock(&mut self) {
        if self.timing == TimingPolicy::Elapsed {
            self.host_clock
                .pause(&mut self.bus, self.processor.machine_registers_mut());
        }
    }

    #[cfg(feature = "gdb")]
    pub(crate) fn resume_host_clock(&mut self) {
        if self.timing == TimingPolicy::Elapsed {
            self.host_clock.resume();
        }
    }

    pub(crate) fn set_steps_per_tick(
        &mut self,
        steps: u32,
    ) -> Result<(), EmulatorError> {
        self.step_clock.set_period(self.timing, steps)
    }

    pub(crate) fn set_timer_hz(
        &mut self,
        hz: u32,
    ) -> Result<(), EmulatorError> {
        self.host_clock.set_frequency(hz)
    }

    pub(crate) fn step_report(
        &mut self,
    ) -> Result<StepReport<StepOutcome<WordOf<A>>>, EmulatorError> {
        self.bus.check_host_errors()?;
        let prior_completion = self.completion;
        let outcome = self.step()?;
        self.last_outcome = Some(outcome);
        self.bus.check_host_errors()?;
        let completion = self.completion.filter(|_| prior_completion.is_none());
        Ok(StepReport {
            completion,
            outcome,
        })
    }
}
