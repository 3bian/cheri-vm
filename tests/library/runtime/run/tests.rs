// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use crate::hart::execution::{StopReason, TrapPolicy};
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::uart::Uart;

use crate::runtime::output::OutputPresentation;
use crate::runtime::run::{GuestOutput, RunOptions, stop_before_step};

#[cfg(unix)]
mod polling {
    use std::io::Write;

    use super::options;
    use crate::hart::execution::{ExecutionReport, StepReport, StopReason};
    use crate::machine::EmulatorError;
    use crate::machine::bus::Bus;
    use crate::machine::completion::HostInterface;
    use crate::machine::devices::uart::Uart;
    use crate::machine::host::uart::input::UartInput;
    use crate::machine::timing_policy::STEPS_PER_HOST_POLL;
    use crate::runtime::output::{GuestOutput, OutputPresentation};
    use crate::runtime::run::{RunOptions, RunTarget, run_with_options};

    const UART: u32 = 0x1000;

    struct ScriptedTarget {
        arrival: Option<u64>,
        bus: Bus,
        host: Option<HostInterface>,
        last_step: u64,
        steps: u64,
        trap_at_end: bool,
        waiting: bool,
        writer: std::fs::File,
    }

    impl ScriptedTarget {
        fn has_input(&self) -> bool {
            self.bus
                .device::<Uart>("uart0")
                .unwrap()
                .receive_remaining()
                < Uart::RECEIVE_CAPACITY
        }
    }

    impl RunTarget for ScriptedTarget {
        type Outcome = bool;

        fn stack_pointer(&self) -> u64 {
            0
        }

        fn position(&self) -> (u64, u64) {
            (0x8000_0000, self.steps)
        }

        fn execution_report(&self, stop: StopReason) -> ExecutionReport {
            ExecutionReport {
                attempted_steps: self.steps,
                fetched_instructions: self.steps,
                retired_instructions: self.steps,
                stop,
                virtual_time: 0,
            }
        }

        fn handle_step(
            &mut self,
            _: &bool,
            _: &RunOptions,
            _: &mut Vec<u8>,
            _: &mut GuestOutput<impl Write>,
            _: &mut u64,
        ) -> Result<Option<StopReason>, EmulatorError> {
            Ok(None)
        }

        fn io_parts(&mut self) -> (&mut Bus, &mut Option<HostInterface>) {
            (&mut self.bus, &mut self.host)
        }

        fn is_waiting(&self, outcome: &bool) -> bool {
            *outcome
        }

        fn present_state(
            &self,
            _: &RunOptions,
            _: &mut GuestOutput<impl Write>,
        ) -> Result<(), EmulatorError> {
            Ok(())
        }

        fn handle_error(
            &mut self,
            _: EmulatorError,
            _: &mut GuestOutput<impl Write>,
            _: OutputPresentation,
        ) -> Result<StopReason, EmulatorError> {
            Ok(StopReason::GuestEbreak {
                address: 0x8000_0000,
            })
        }

        fn step_report(&mut self) -> Result<StepReport<bool>, EmulatorError> {
            self.steps += 1;
            if self.steps == 1 {
                self.writer.write_all(b"Q")?;
            }
            if self.arrival.is_none() && self.has_input() {
                self.arrival = Some(self.steps);
            }
            if self.steps == self.last_step && self.trap_at_end {
                self.bus.write(UART.into(), 4, u32::from(b'!'))?;
                return Err(EmulatorError::Execution("guest trapped".into()));
            }
            Ok(StepReport {
                outcome: self.waiting,
                completion: (self.steps == self.last_step).then_some(0),
            })
        }
    }

    fn run(
        waiting: bool,
        last_step: u64,
        trap_at_end: bool,
    ) -> (ScriptedTarget, StopReason) {
        let directory =
            crate::program::temporary::TemporaryDirectory::new().unwrap();
        let path = directory.path().join("uart");
        nix::unistd::mkfifo(&path, nix::sys::stat::Mode::S_IRWXU).unwrap();
        let mut bus = Bus::new();
        bus.add_device("uart0".into(), UART.into(), 256, Box::new(Uart::new()))
            .unwrap();
        let input = UartInput::open(&bus, "uart0", &path).unwrap();
        let target = ScriptedTarget {
            arrival: None,
            bus,
            host: None,
            last_step,
            steps: 0,
            trap_at_end,
            waiting,
            writer: std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap(),
        };
        let mut options = options();
        options.steps = None;
        options.uart.inputs.push(input);
        let (target, report) = run_with_options(target, options).unwrap();
        (target, report.stop)
    }

    #[test]
    fn input_reaches_a_busy_guest_within_one_poll() {
        let (target, stop) = run(false, 2 * STEPS_PER_HOST_POLL, false);
        assert_eq!(stop, StopReason::GuestCompletion { code: 0 });
        assert_eq!(target.arrival, Some(STEPS_PER_HOST_POLL + 1));
    }

    #[test]
    fn input_reaches_a_waiting_guest_at_once() {
        let (target, stop) = run(true, STEPS_PER_HOST_POLL / 2, false);
        assert_eq!(stop, StopReason::GuestCompletion { code: 0 });
        assert_eq!(target.arrival, Some(2));
    }

    #[test]
    fn output_written_just_before_a_trap_that_ends_the_run_is_printed() {
        let (target, stop) = run(false, 10, true);
        assert_eq!(
            stop,
            StopReason::GuestEbreak {
                address: 0x8000_0000
            }
        );
        assert!(
            target
                .bus
                .device::<Uart>("uart0")
                .unwrap()
                .transmitted()
                .is_empty()
        );
    }
}

#[test]
fn breakpoint_precedes_the_budget_without_advancing_the_target() {
    let mut options = options();
    options.breakpoint = vec![0x8000_0004];
    let mut bytes = Vec::new();
    let mut output = GuestOutput::new(&mut bytes);
    let stop = stop_before_step(0x8000_0004, 1, &options, Some(1), &mut output)
        .unwrap();
    assert_eq!(
        stop,
        Some(StopReason::Breakpoint {
            address: 0x8000_0004
        })
    );
    output.flush().unwrap();
}

fn options() -> RunOptions {
    RunOptions {
        web: None,
        insights: None,
        breakpoint: Vec::new(),
        memory_dump: None,
        presentation: OutputPresentation::Raw,
        exit_line: crate::runtime::request::ExitLine::Shown,
        registers: None,
        steps: Some(1),
        trace: false,
        trap_policy: TrapPolicy::Vector,
        uart: Default::default(),
        websocket: Default::default(),
    }
}

#[test]
fn output_is_drained_and_the_original_failure_takes_precedence() {
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut bus = Bus::new();
    bus.add_device("uart0".into(), 0x1000, 256, Box::new(Uart::new()))
        .unwrap();
    bus.write(0x1000, 4, u32::from(b'X')).unwrap();
    let mut output = GuestOutput::new(BrokenWriter);
    let failure = EmulatorError::Execution("original failure".into());
    let result = crate::runtime::run::runtime_io::finish_output(
        Err(failure),
        &mut bus,
        &mut None,
        &mut Vec::new(),
        &mut output,
        OutputPresentation::Raw,
    );
    assert!(result.unwrap_err().to_string().contains("original failure"));
    assert!(
        bus.device::<Uart>("uart0")
            .unwrap()
            .transmitted()
            .is_empty()
    );
}
