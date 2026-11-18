// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::hart::execution::{StopReason, TrapPolicy};
use crate::hart::{CheriotRv32e, Hart, StepOutcome};
use crate::isa::decode::DecodedInstruction;
use crate::isa::pointer::cheriot::Capability;
use crate::machine::EmulatorError;
use crate::runtime::options::run::RunOptions;
use crate::runtime::output::{GuestOutput, OutputPresentation};
use crate::runtime::register::Register;
use crate::runtime::run::hart::Presentation;
use crate::runtime::runtime_io;

impl Presentation for CheriotRv32e {
    const FLUSH_WHEN_TRACING: bool = true;
    const REPORT_TRAPS: bool = true;
    const STOPS_IN_STEP: bool = false;

    fn handle_step(
        hart: &mut Hart<Self>,
        outcome: &StepOutcome,
        options: &RunOptions,
        scratch: &mut Vec<u8>,
        output: &mut GuestOutput<impl Write>,
        traps: &mut u64,
    ) -> Result<Option<StopReason>, EmulatorError> {
        match *outcome {
            StepOutcome::Retired { address, decoded } if options.trace => {
                let encoded = decoded.encoded;
                output.diagnostic(
                    &format!("0x{address:08x} [0x{encoded:08x}]\n"),
                    options.presentation,
                )?;
            }
            StepOutcome::Trapped {
                address,
                cause,
                interrupt,
                value,
            } => {
                let cause = cause | (u32::from(interrupt) << 31);
                *traps = traps.saturating_add(1);
                if options.trace {
                    output.diagnostic(
                        &format!(
                            "Trap at 0x{address:08x}: cause 0x{cause:08x}, \
                             value 0x{value:08x}\n"
                        ),
                        options.presentation,
                    )?;
                }
                if options.trap_policy == TrapPolicy::Stop {
                    if cause == 3 {
                        runtime_io::drain_output(
                            &mut hart.bus,
                            &mut hart.host_interface,
                            scratch,
                            output,
                            options.presentation,
                        )?;
                        output.flush()?;
                        output.diagnostic(
                            &format!("Guest EBREAK at 0x{address:08x}\n"),
                            options.presentation,
                        )?;
                        return Ok(Some(StopReason::GuestEbreak {
                            address: address.into(),
                        }));
                    }
                    return Err(EmulatorError::CheriotTrap {
                        address: address.into(),
                        cause,
                        value: value.into(),
                    });
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn is_wait(decoded: &DecodedInstruction) -> bool {
        decoded.encoded == 0x1050_0073
    }

    fn present_state(
        hart: &Hart<Self>,
        options: &RunOptions,
        output: &mut GuestOutput<impl Write>,
    ) -> Result<(), EmulatorError> {
        if let Some(selection) = &options.registers {
            let processor = &hart.processor;
            if selection.includes(Register::Pc) {
                output.diagnostic(
                    &format!(
                        "Program counter: 0x{:08x}\n",
                        processor.program_counter
                    ),
                    options.presentation,
                )?;
            }
            for (index, capability) in processor.registers().iter().enumerate()
            {
                if selection.includes(Register::Capability(index as u8)) {
                    print_capability(
                        &format!("c{index:02}"),
                        *capability,
                        output,
                        options.presentation,
                    )?;
                }
            }
            for (register, name, capability) in [
                (Register::Pcc, "PCC", processor.pcc),
                (Register::Mtcc, "MTCC", processor.machine_trap),
                (Register::Mtdc, "MTDC", processor.machine_data),
                (Register::Mscratchc, "MScratchC", processor.machine_scratch),
                (Register::Mepcc, "MEPCC", processor.machine_exception),
            ] {
                if selection.includes(register) {
                    print_capability(
                        name,
                        capability,
                        output,
                        options.presentation,
                    )?;
                }
            }
        }
        Ok(())
    }
}

fn print_capability(
    name: &str,
    capability: Capability,
    output: &mut GuestOutput<impl Write>,
    presentation: OutputPresentation,
) -> Result<(), EmulatorError> {
    let (payload, tag) = capability.to_memory();
    let (base, top) = capability.bounds();
    output.diagnostic(
        &format!(
            "{name}: address=0x{:08x} payload=0x{payload:016x} tag={} \
             base=0x{base:08x} top=0x{top:09x} permissions=0x{:03x} type={}\n",
            capability.address(),
            u8::from(tag),
            capability.permissions(),
            capability.object_type()
        ),
        presentation,
    )?;
    Ok(())
}
