// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::machine_registers::MachineRegisters;
use crate::machine::bus::Bus;

mod host_clock;
mod step_clock;

pub(crate) use host_clock::HostClock;
pub(crate) use step_clock::StepClock;

pub(crate) const DEFAULT_STEPS_PER_TICK: u32 = 1;
pub(crate) const DEFAULT_TIMER_HZ: u32 = 2_000;
pub(crate) const MAX_TIMER_HZ: u32 = 1_000_000_000;
pub(crate) const STEPS_PER_HOST_POLL: u64 = 1024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimingPolicy {
    #[default]
    Elapsed,
    PerStep,
}

impl TimingPolicy {
    #[inline(always)]
    pub(crate) fn advance<W>(
        self,
        bus: &mut Bus,
        registers: &mut MachineRegisters<W>,
        clock: &mut StepClock,
        inhibit_before: u32,
        writes_cycle: bool,
    ) {
        synchronize_machine_time(bus, registers);
        let tick = clock.advance();
        if self == Self::PerStep && !tick {
            return;
        }
        if inhibit_before & 1 == 0 && !writes_cycle {
            registers.cycle = registers.cycle.wrapping_add(1);
        }
        if self == Self::PerStep {
            registers.time = registers.time.wrapping_add(1);
            bus.tick_devices(1);
        }
    }

    pub(crate) fn wait_for_interrupt<W>(
        self,
        bus: &mut Bus,
        registers: &mut MachineRegisters<W>,
        clock: &mut HostClock,
    ) {
        match self {
            Self::PerStep => (),
            Self::Elapsed => {
                bus.poll_host_inputs();
                refresh_interrupts(bus, registers);
                if ![
                    crate::machine::devices::InterruptLine::MachineExternal,
                    crate::machine::devices::InterruptLine::MachineSoftware,
                    crate::machine::devices::InterruptLine::MachineTimer,
                    crate::machine::devices::InterruptLine::SupervisorExternal,
                ]
                .into_iter()
                .any(|line| bus.interrupt_pending(line))
                {
                    std::thread::sleep(clock.wait_quantum());
                }
                clock.synchronize(bus, registers);
                bus.poll_host_inputs();
                refresh_interrupts(bus, registers);
            }
        }
    }
}

fn refresh_interrupts<W>(bus: &mut Bus, registers: &mut MachineRegisters<W>) {
    bus.refresh_interrupts();
    registers.interrupt_pending =
        (registers.interrupt_pending & 0x222) | bus.machine_interrupt_bits();
}

#[inline]
pub(crate) fn synchronize_machine_time<W>(
    bus: &mut Bus,
    registers: &mut MachineRegisters<W>,
) {
    if let Some(time) = bus.take_machine_time() {
        registers.time = time;
    }
}

pub(crate) fn validate_step_period(
    timing: TimingPolicy,
    period: u32,
) -> Result<(), crate::machine::EmulatorError> {
    if period == 0 || (timing == TimingPolicy::Elapsed && period != 1) {
        return Err(crate::machine::EmulatorError::Configuration(
            "steps_per_tick must be positive and requires per-step timing \
             when greater than 1"
                .into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_timer_hz(
    hz: u32,
) -> Result<(), crate::machine::EmulatorError> {
    if !(1..=MAX_TIMER_HZ).contains(&hz) {
        return Err(crate::machine::EmulatorError::Configuration(format!(
            "timer_hz must be between 1 and {MAX_TIMER_HZ}"
        )));
    }
    Ok(())
}
