// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use crate::hart::execution::{ExecutionReport, StopReason};
use crate::machine::EmulatorError;
use crate::machine::timing_policy::STEPS_PER_HOST_POLL;

pub(crate) use crate::runtime::options::run::RunOptions;
pub(crate) use crate::runtime::output::GuestOutput;
pub(crate) use crate::runtime::run::run_target::RunTarget;
pub(crate) use crate::runtime::runtime_io;

#[cfg(all(feature = "gdb", unix))]
pub(crate) mod debugger;
pub(crate) mod hart;
mod run_target;

fn poll_host<T: RunTarget, W: Write>(
    target: &mut T,
    options: &mut RunOptions,
    scratch: &mut Vec<u8>,
    output: &mut GuestOutput<W>,
) -> Result<(), EmulatorError> {
    poll_uart_inputs(target, options)?;
    options.websocket.settle()?;
    let presentation = options.presentation;
    let (bus, host_interface) = target.io_parts();
    runtime_io::drain_output(bus, host_interface, scratch, output, presentation)
}

fn poll_uart_inputs<T: RunTarget>(
    target: &mut T,
    options: &mut RunOptions,
) -> Result<(), EmulatorError> {
    for input in &mut options.uart.inputs {
        input.poll(target.io_parts().0)?;
    }
    for link in &mut options.uart.links {
        link.poll(target.io_parts().0)?;
    }
    Ok(())
}

fn run_to_stop<T: RunTarget>(
    mut target: T,
    mut options: RunOptions,
) -> Result<(T, ExecutionReport, u64), EmulatorError> {
    let maximum_steps = options.steps.filter(|limit| *limit != 0);
    let mut scratch = Vec::new();
    let mut traps = 0_u64;
    let mut output = GuestOutput::new(io::stdout().lock())
        .with_uart_outputs(std::mem::take(&mut options.uart.outputs))
        .with_uart_links(std::mem::take(&mut options.uart.link_outputs));
    let presentation = options.presentation;
    let result = (|| -> Result<StopReason, EmulatorError> {
        loop {
            let (address, attempted_steps) = target.position();
            if let Some(stop) = stop_before_step(
                address,
                attempted_steps,
                &options,
                maximum_steps,
                &mut output,
            )? {
                return Ok(stop);
            }
            if let Some(insights) = &options.insights {
                insights
                    .borrow_mut()
                    .record(address as u32, target.stack_pointer() as u32);
            }
            let report = match target.step_report() {
                Ok(report) => report,
                Err(error) => {
                    return target.handle_error(
                        error,
                        &mut output,
                        presentation,
                    );
                }
            };
            let waiting = target.is_waiting(&report.outcome);
            if waiting {
                poll_uart_inputs(&mut target, &mut options)?;
            }
            if let Some(stop) = target.handle_step(
                &report.outcome,
                &options,
                &mut scratch,
                &mut output,
                &mut traps,
            )? {
                return Ok(stop);
            }
            if target.position().1.is_multiple_of(STEPS_PER_HOST_POLL)
                || waiting
                || report.completion.is_some()
            {
                poll_host(
                    &mut target,
                    &mut options,
                    &mut scratch,
                    &mut output,
                )?;
                if waiting || (T::FLUSH_WHEN_TRACING && options.trace) {
                    output.flush()?;
                }
            }
            if let Some(code) = report.completion {
                return Ok(StopReason::GuestCompletion { code });
            }
        }
    })();
    let (bus, host_interface) = target.io_parts();
    let stopped = runtime_io::finish_output(
        result,
        bus,
        host_interface,
        &mut scratch,
        &mut output,
        presentation,
    );
    let restored = options.uart.restore();
    let stop = stopped?;
    restored?;
    target.present_state(&options, &mut output)?;
    if let Some((start, count)) = options.memory_dump {
        runtime_io::print_memory(
            target.io_parts().0,
            start,
            count,
            &mut output,
            presentation,
        )?;
    }
    output.flush()?;
    let report = target.execution_report(stop);
    Ok((target, report, traps))
}

pub(crate) fn run_with_options<T: RunTarget>(
    target: T,
    mut options: RunOptions,
) -> Result<(T, ExecutionReport), EmulatorError> {
    let web = options.web.take();
    let websocket = options.websocket.handle();
    let exit_line = options.exit_line;
    let outcome = run_to_stop(target, options);
    websocket.finish(outcome.as_ref().map(|(_, report, _)| report));
    drop(web);
    let (target, report, traps) = outcome?;
    runtime_io::finish_report(
        &report,
        target.position().0,
        T::REPORT_TRAPS.then_some(traps),
        exit_line,
    )?;
    Ok((target, report))
}

#[inline(always)]
pub(crate) fn stop_before_step(
    address: u64,
    attempted_steps: u64,
    options: &RunOptions,
    maximum_steps: Option<u64>,
    output: &mut GuestOutput<impl Write>,
) -> Result<Option<StopReason>, EmulatorError> {
    if options.breakpoint.contains(&address) {
        output.diagnostic(
            &format!("Breakpoint at 0x{address:08x}\n"),
            options.presentation,
        )?;
        return Ok(Some(StopReason::Breakpoint { address }));
    }
    Ok(maximum_steps
        .filter(|limit| attempted_steps >= *limit)
        .map(|_| StopReason::BudgetExhausted))
}
