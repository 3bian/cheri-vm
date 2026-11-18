// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use gdbstub::arch::Arch;
use gdbstub::common::Signal;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::single_register_access::{
    SingleRegisterAccess, SingleRegisterAccessOps,
};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps,
    SingleThreadSingleStep, SingleThreadSingleStepOps,
};
use gdbstub::target::ext::breakpoints::{
    Breakpoints, BreakpointsOps, HwBreakpoint, HwBreakpointOps, SwBreakpoint,
    SwBreakpointOps,
};
use gdbstub::target::ext::monitor_cmd::{
    ConsoleOutput, MonitorCmd, MonitorCmdOps,
};
use gdbstub::target::ext::target_description_xml_override::{
    TargetDescriptionXmlOverride, TargetDescriptionXmlOverrideOps,
};
use gdbstub::target::{Target as GdbTarget, TargetError, TargetResult};

use super::{
    BreakpointType, Core, RunControl, RunState, SessionTarget, StopReason,
};
use crate::machine::EmulatorError;

type AddressOf<C> = <<C as Core>::Arch as Arch>::Usize;

pub(crate) struct Target<C: Core> {
    control: RunControl,
    pub(crate) core: C,
    description: Option<String>,
    fault: Option<C::Fault>,
    pub reason: StopReason,
    pub state: RunState,
}

impl<C: Core> Target<C> {
    pub(crate) fn advance(
        &mut self,
    ) -> Result<Option<StopReason>, EmulatorError> {
        if self.state == RunState::Halted {
            return Ok(None);
        }
        let address = self.core.program_counter();
        if let Some(reason) = self
            .control
            .before_step(address, self.core.attempted_steps())
        {
            return Ok(Some(self.halt(reason)));
        }
        let stepped = self.core.step()?;
        if let Some(trap) = stepped.trap {
            self.fault = Some(trap.fault);
            if let Some(reason) =
                StopReason::from_fault(trap.address, trap.cause, trap.interrupt)
            {
                return Ok(Some(self.halt(reason)));
            }
        }
        if let Some(code) = stepped.completion {
            return Ok(Some(self.halt(StopReason::Completed(code))));
        }
        if self.state == RunState::SingleStep {
            return Ok(Some(self.halt(StopReason::Step)));
        }
        Ok(None)
    }

    fn breakpoint(
        &mut self,
        address: AddressOf<C>,
        kind: usize,
        breakpoint: BreakpointType,
        insert: bool,
    ) -> TargetResult<bool, Self> {
        let address = address.into();
        let alignment = self.core.minimum_alignment();
        let done = if insert {
            let core = &self.core;
            self.control.insert_breakpoint(
                |address, kind| core.accepts_breakpoint(address, kind),
                breakpoint,
                address,
                kind,
                alignment,
            )
        } else {
            self.control
                .remove_breakpoint(breakpoint, address, kind, alignment)
        };
        if done {
            Ok(true)
        } else {
            Err(TargetError::NonFatal)
        }
    }

    pub(crate) fn halt(&mut self, reason: StopReason) -> StopReason {
        self.core.pause_host_clock();
        self.state = RunState::Halted;
        self.reason = reason;
        reason
    }

    pub(crate) fn inspect(
        &self,
        address: u64,
        data: &mut [u8],
    ) -> Result<(), EmulatorError> {
        self.core.inspect(address, data)
    }

    pub(crate) fn into_core(mut self) -> C {
        self.core.resume_host_clock();
        self.core
    }

    pub(crate) fn monitor(
        &self,
        command: &[u8],
    ) -> Result<String, &'static str> {
        let words = super::monitor_words(command)?;
        match words.as_slice() {
            ["help"] => Ok(C::MONITOR_HELP.into()),
            ["fault"] => Ok(match &self.fault {
                Some(fault) => C::describe_fault(fault),
                None => "no trap observed\n".into(),
            }),
            ["status"] => {
                Ok(format!("{:?}; {}\n", self.reason, self.core.status()))
            }
            _ => self
                .core
                .monitor(&words)
                .unwrap_or(Err("unknown monitor command; use monitor help")),
        }
    }

    pub(crate) fn new(mut core: C, maximum_steps: Option<u64>) -> Self {
        core.pause_host_clock();
        Self {
            control: RunControl::new(maximum_steps),
            description: core.target_description(),
            fault: None,
            reason: StopReason::Startup,
            state: RunState::Halted,
            core,
        }
    }

    fn resume_with(
        &mut self,
        state: RunState,
        signal: Option<Signal>,
    ) -> Result<(), EmulatorError> {
        self.control
            .resume(&mut self.state, self.reason, state, signal)?;
        self.core.resume_host_clock();
        Ok(())
    }
}

impl<C: Core> Breakpoints for Target<C> {
    fn support_hw_breakpoint(&mut self) -> Option<HwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl<C: Core> GdbTarget for Target<C> {
    type Arch = C::Arch;
    type Error = EmulatorError;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }

    fn support_monitor_cmd(&mut self) -> Option<MonitorCmdOps<'_, Self>> {
        Some(self)
    }

    fn support_target_description_xml_override(
        &mut self,
    ) -> Option<TargetDescriptionXmlOverrideOps<'_, Self>> {
        self.description.is_some().then_some(self)
    }

    fn use_rle(&self) -> bool {
        false
    }
}

impl<C: Core> HwBreakpoint for Target<C> {
    fn add_hw_breakpoint(
        &mut self,
        address: AddressOf<C>,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        self.breakpoint(address, kind, BreakpointType::Hardware, true)
    }

    fn remove_hw_breakpoint(
        &mut self,
        address: AddressOf<C>,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        self.breakpoint(address, kind, BreakpointType::Hardware, false)
    }
}

impl<C: Core> MonitorCmd for Target<C> {
    fn handle_monitor_cmd(
        &mut self,
        command: &[u8],
        output: ConsoleOutput<'_>,
    ) -> Result<(), Self::Error> {
        present_monitor(self.monitor(command), output);
        Ok(())
    }
}

impl<C: Core> SessionTarget for Target<C> {
    fn advance(&mut self) -> Result<Option<StopReason>, EmulatorError> {
        Self::advance(self)
    }

    fn drain_output_each(
        &mut self,
        scratch: &mut Vec<u8>,
        each: &mut impl FnMut(&str, &str, &[u8]) -> Result<(), EmulatorError>,
    ) -> Result<(), EmulatorError> {
        self.core.drain_output_each(scratch, each)
    }

    fn halt(&mut self, reason: StopReason) -> StopReason {
        Self::halt(self, reason)
    }

    fn poll_input(
        &mut self,
        input: &mut crate::machine::host::uart::input::UartInput,
    ) -> Result<(), EmulatorError> {
        self.core.poll_input(input)
    }
}

impl<C: Core> SingleRegisterAccess<()> for Target<C> {
    fn read_register(
        &mut self,
        _: (),
        id: <C::Arch as Arch>::RegId,
        bytes: &mut [u8],
    ) -> TargetResult<usize, Self> {
        self.core
            .read_register(id, bytes)
            .ok_or(TargetError::NonFatal)
    }

    fn write_register(
        &mut self,
        _: (),
        _: <C::Arch as Arch>::RegId,
        _: &[u8],
    ) -> TargetResult<(), Self> {
        Err(TargetError::NonFatal)
    }
}

impl<C: Core> SingleThreadBase for Target<C> {
    fn read_addrs(
        &mut self,
        address: AddressOf<C>,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        self.inspect(address.into(), data)
            .map_err(|_| TargetError::NonFatal)?;
        Ok(data.len())
    }

    fn read_registers(
        &mut self,
        registers: &mut <C::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        *registers = self.core.registers();
        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    fn support_single_register_access(
        &mut self,
    ) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }

    fn write_addrs(
        &mut self,
        _: AddressOf<C>,
        _: &[u8],
    ) -> TargetResult<(), Self> {
        Err(TargetError::NonFatal)
    }

    fn write_registers(
        &mut self,
        _: &<C::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        Err(TargetError::NonFatal)
    }
}

impl<C: Core> SingleThreadResume for Target<C> {
    fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume_with(RunState::Continuing, signal)
    }

    fn support_single_step(
        &mut self,
    ) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl<C: Core> SingleThreadSingleStep for Target<C> {
    fn step(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume_with(RunState::SingleStep, signal)
    }
}

impl<C: Core> SwBreakpoint for Target<C> {
    fn add_sw_breakpoint(
        &mut self,
        address: AddressOf<C>,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        self.breakpoint(address, kind, BreakpointType::Software, true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        address: AddressOf<C>,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        self.breakpoint(address, kind, BreakpointType::Software, false)
    }
}

impl<C: Core> TargetDescriptionXmlOverride for Target<C> {
    fn target_description_xml(
        &self,
        annex: &[u8],
        offset: u64,
        length: usize,
        buffer: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let Some(description) = self
            .description
            .as_deref()
            .filter(|_| annex == b"target.xml")
        else {
            return Err(TargetError::NonFatal);
        };
        let bytes = description.as_bytes();
        let start = usize::try_from(offset)
            .map_or(bytes.len(), |start| start.min(bytes.len()));
        let part = &bytes[start..bytes.len().min(start.saturating_add(length))];
        let count = part.len().min(buffer.len());
        buffer[..count].copy_from_slice(&part[..count]);
        Ok(count)
    }
}

fn present_monitor(
    result: Result<String, &'static str>,
    mut output: ConsoleOutput<'_>,
) {
    match result {
        Ok(text) => gdbstub::output!(output, "{}", text),
        Err(error) => gdbstub::outputln!(output, "error: {}", error),
    }
}
