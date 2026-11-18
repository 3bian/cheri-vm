// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::hart::execution::StopReason;
use crate::hart::plain::Plain;
use crate::hart::plain::debug::{DebugModel, describe_float};
use crate::hart::privilege::PrivilegeModel;
use crate::hart::{Hart, StepOutcome};
use crate::isa::decode::DecodedInstruction;
use crate::isa::instruction::Instruction;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::EmulatorError;
use crate::machine::ExceptionCause;
use crate::runtime::options::run::RunOptions;
use crate::runtime::output::{GuestOutput, OutputPresentation};
use crate::runtime::register::Register;
use crate::runtime::run::hart::Presentation;

impl<X: Xlen, P: PrivilegeModel, M: DebugModel<X>> Presentation
    for Plain<X, P, M>
{
    const FLUSH_WHEN_TRACING: bool = false;
    const REPORT_TRAPS: bool = false;
    const STOPS_IN_STEP: bool = true;

    fn handle_error(
        error: EmulatorError,
        output: &mut GuestOutput<impl Write>,
        presentation: OutputPresentation,
    ) -> Result<StopReason, EmulatorError> {
        match error {
            EmulatorError::Trap {
                cause: ExceptionCause::Breakpoint,
                address,
                ..
            } => {
                let digits = digits::<X>();
                output.diagnostic(
                    &format!("Guest EBREAK at 0x{address:0digits$x}\n"),
                    presentation,
                )?;
                Ok(StopReason::GuestEbreak { address })
            }
            error => Err(error),
        }
    }

    #[inline]
    fn handle_step(
        _hart: &mut Hart<Self>,
        outcome: &StepOutcome<X::Word>,
        options: &RunOptions,
        _scratch: &mut Vec<u8>,
        output: &mut GuestOutput<impl Write>,
        _traps: &mut u64,
    ) -> Result<Option<StopReason>, EmulatorError> {
        match outcome {
            StepOutcome::Retired { .. } if !options.trace => Ok(None),
            StepOutcome::Trapped { .. } if P::SUPERVISOR && !options.trace => {
                Ok(None)
            }
            outcome => present_step::<X>(outcome, options, output),
        }
    }

    fn is_wait(decoded: &DecodedInstruction) -> bool {
        decoded.instruction == Instruction::WaitForInterrupt
    }

    fn present_state(
        hart: &Hart<Self>,
        options: &RunOptions,
        output: &mut GuestOutput<impl Write>,
    ) -> Result<(), EmulatorError> {
        let digits = digits::<X>();
        output.diagnostic(
            &format!(
                "PC=0x{:0digits$x} retired={} cycles={} virtual_ticks={}\n",
                hart.processor.program_counter,
                hart.processor.machine_registers.retired,
                hart.processor.machine_registers.cycle,
                hart.processor.machine_registers.time
            ),
            options.presentation,
        )?;
        if let Some(selection) = &options.registers {
            let processor = &hart.processor;
            for (index, value) in processor.registers().iter().enumerate() {
                if selection.includes(Register::Integer(index as u8)) {
                    let line = if M::CAPABILITIES {
                        let whole = M::describe_register(
                            &processor.shadow,
                            index as u8,
                            *value,
                        );
                        format!("x{index:02}: {whole}\n")
                    } else {
                        format!(
                            "x{index:02}: 0x{value:0digits$x} ({})\n",
                            value.to_i64()
                        )
                    };
                    output.diagnostic(&line, options.presentation)?;
                }
            }
            if selection.includes(Register::Pcc) {
                let whole = M::describe_program_counter(
                    &processor.shadow,
                    processor.program_counter,
                );
                output.diagnostic(
                    &format!("pcc: {whole}\n"),
                    options.presentation,
                )?;
            }
            for (index, bits) in processor.float_registers.iter().enumerate() {
                if selection.includes(Register::Float(index as u8)) {
                    let value = describe_float(processor.extensions, *bits);
                    output.diagnostic(
                        &format!("f{index:02}: {value}\n"),
                        options.presentation,
                    )?;
                }
            }
            if selection.includes(Register::FloatControl) {
                output.diagnostic(
                    &format!("fcsr: 0x{:02x}\n", processor.float_control),
                    options.presentation,
                )?;
            }
        }
        Ok(())
    }
}

fn digits<X: Xlen>() -> usize {
    (X::BITS / 4) as usize
}

#[inline(never)]
fn present_step<X: Xlen>(
    outcome: &StepOutcome<X::Word>,
    options: &RunOptions,
    output: &mut GuestOutput<impl Write>,
) -> Result<Option<StopReason>, EmulatorError> {
    let digits = digits::<X>();
    match outcome {
        StepOutcome::Retired { address, decoded } => {
            output.diagnostic(
                &format!(
                    "0x{address:0digits$x} [{:04x}/{}]: {:?}\n",
                    decoded.encoded,
                    decoded.length(),
                    decoded.instruction
                ),
                options.presentation,
            )?;
        }
        StepOutcome::Trapped {
            address,
            cause,
            interrupt,
            ..
        } => {
            output.diagnostic(
                &format!(
                    "{} trap cause {cause} at 0x{address:0digits$x}\n",
                    if *interrupt { "interrupt" } else { "exception" }
                ),
                options.presentation,
            )?;
        }
    }
    Ok(None)
}
