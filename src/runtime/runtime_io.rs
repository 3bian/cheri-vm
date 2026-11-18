// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::hart::execution::{ExecutionReport, StopReason};
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::completion::HostInterface;

use crate::runtime::output::GuestOutput;
use crate::runtime::output::OutputPresentation;

pub(crate) fn drain_output(
    bus: &mut Bus,
    host_interface: &mut Option<HostInterface>,
    scratch: &mut Vec<u8>,
    output: &mut GuestOutput<impl Write>,
    presentation: OutputPresentation,
) -> Result<(), EmulatorError> {
    crate::runtime::host_output::drain(
        bus,
        host_interface,
        scratch,
        |kind, name, bytes| {
            output
                .present(kind, name, bytes, presentation)
                .map_err(EmulatorError::from)
        },
    )?;
    output.write(&[])?;
    Ok(())
}

pub(crate) fn finish_output(
    result: Result<StopReason, EmulatorError>,
    bus: &mut Bus,
    host_interface: &mut Option<HostInterface>,
    scratch: &mut Vec<u8>,
    output: &mut GuestOutput<impl Write>,
    presentation: OutputPresentation,
) -> Result<StopReason, EmulatorError> {
    let drain =
        drain_output(bus, host_interface, scratch, output, presentation)
            .and_then(|()| output.flush().map_err(EmulatorError::from));
    let stop = result?;
    drain?;
    Ok(stop)
}

pub(crate) fn finish_report(
    report: &ExecutionReport,
    address: u64,
    traps: Option<u64>,
    exit_line: crate::runtime::request::ExitLine,
) -> Result<(), EmulatorError> {
    match report.stop {
        StopReason::BudgetExhausted => Err(EmulatorError::Execution(format!(
            "step limit reached after {} steps at program counter \
             0x{address:08x}; the guest has not completed. Increase --steps, \
             omit the limit, or use --steps 0 for unlimited execution",
            report.attempted_steps
        ))),
        StopReason::GuestCompletion { code } => {
            let traps = traps
                .map(|count| format!("; traps: {count}"))
                .unwrap_or_default();
            if exit_line == crate::runtime::request::ExitLine::Shown {
                writeln!(
                    std::io::stderr().lock(),
                    "Guest exit status: {code}; steps: {}{traps}",
                    report.attempted_steps
                )?;
            }

            if code == 0 {
                Ok(())
            } else {
                Err(EmulatorError::Execution(format!(
                    "guest exited with status {code}"
                )))
            }
        }
        _ => Ok(()),
    }
}

pub(crate) fn print_memory(
    bus: &Bus,
    start: u64,
    count: usize,
    output: &mut GuestOutput<impl Write>,
    presentation: OutputPresentation,
) -> Result<(), EmulatorError> {
    output.diagnostic(&format!("Memory at 0x{start:08x}:\n"), presentation)?;
    for offset in 0..count {
        let address = start.checked_add(offset as u64).ok_or_else(|| {
            EmulatorError::Configuration("memory dump address overflow".into())
        })?;
        let byte = bus.inspect(address, 1)?;
        if offset % 16 == 0 {
            output.diagnostic(&format!("0x{address:08x}:"), presentation)?;
        }
        output.diagnostic(&format!(" {byte:02x}"), presentation)?;
        if offset % 16 == 15 || offset + 1 == count {
            output.diagnostic("\n", presentation)?;
        }
    }
    Ok(())
}
