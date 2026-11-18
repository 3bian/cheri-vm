// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

use crate::hart::machine_registers::MachineRegisters;
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::timing_policy::{
    DEFAULT_TIMER_HZ, refresh_interrupts, validate_timer_hz,
};

const MAX_WAIT_NANOS: u128 = 1_000_000;
const NANOS_PER_SECOND: u128 = 1_000_000_000;

pub(crate) struct HostClock {
    hz: u32,
    last: Option<Instant>,
    paused: bool,
    remainder: u128,
    started: bool,
    #[cfg(feature = "verification")]
    source: Option<Box<dyn Fn() -> Instant>>,
}

impl HostClock {
    fn now(&self) -> Instant {
        #[cfg(feature = "verification")]
        if let Some(source) = &self.source {
            return source();
        }
        Instant::now()
    }

    pub(crate) fn pause<W>(
        &mut self,
        bus: &mut Bus,
        registers: &mut MachineRegisters<W>,
    ) {
        self.synchronize(bus, registers);
        self.last = None;
        self.paused = true;
    }

    pub(crate) fn reset(&mut self) {
        if self.last.is_some() || self.paused {
            self.last = None;
            self.paused = false;
            self.remainder = 0;
        }
    }

    #[cfg(feature = "gdb")]
    pub(crate) fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            self.started = true;
            self.last = Some(self.now());
        }
    }

    pub(crate) fn set_frequency(
        &mut self,
        hz: u32,
    ) -> Result<(), EmulatorError> {
        validate_timer_hz(hz)?;
        if self.started || self.paused {
            return Err(EmulatorError::Configuration(
                "timer frequency cannot change after the clock starts".into(),
            ));
        }
        self.hz = hz;
        Ok(())
    }

    pub(crate) fn synchronize<W>(
        &mut self,
        bus: &mut Bus,
        registers: &mut MachineRegisters<W>,
    ) {
        super::synchronize_machine_time(bus, registers);
        let waited = bus.take_host_wait();
        if self.paused {
            return;
        }
        self.started = true;
        let now = self.now();
        let Some(last) = self.last.replace(now) else {
            return;
        };
        let elapsed = now.duration_since(last).saturating_sub(waited);
        let scaled = elapsed.as_nanos() * u128::from(self.hz) + self.remainder;
        let elapsed_ticks = scaled / NANOS_PER_SECOND;
        self.remainder = scaled % NANOS_PER_SECOND;
        if elapsed_ticks != 0 {
            let ticks = u64::try_from(elapsed_ticks).unwrap_or(u64::MAX);
            registers.time = registers.time.wrapping_add(ticks);
            if ticks != 0 {
                bus.tick_devices(ticks);
            }
            refresh_interrupts(bus, registers);
        }
    }

    pub(crate) fn wait_quantum(&self) -> Duration {
        let nanos = (NANOS_PER_SECOND - self.remainder)
            .div_ceil(u128::from(self.hz))
            .min(MAX_WAIT_NANOS);
        Duration::from_nanos(nanos as u64)
    }
}

impl Default for HostClock {
    fn default() -> Self {
        Self {
            hz: DEFAULT_TIMER_HZ,
            last: None,
            paused: false,
            remainder: 0,
            started: false,
            #[cfg(feature = "verification")]
            source: None,
        }
    }
}
