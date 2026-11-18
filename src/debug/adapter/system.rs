// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::architecture::DebugArchitecture;
use super::super::architecture::Described;
use super::{Advanced, Trapped};
use crate::hart::{Hart, ProcessorState, StepOutcome};
use crate::isa::xlen::{Word, Xlen};
use crate::machine::EmulatorError;

impl<A: DebugArchitecture> System for Hart<A> {
    fn advance(&mut self) -> Result<Advanced, EmulatorError> {
        let exit = Self::advance(self)?;
        let trap = match self.last_outcome {
            Some(StepOutcome::Trapped {
                address,
                cause,
                interrupt,
                ..
            }) => Some(Trapped {
                address: address.to_u64(),
                cause,
                interrupt,
            }),
            _ => None,
        };
        Ok(Advanced { exit, trap })
    }

    fn digits(&self) -> usize {
        (A::Xlen::BITS / 4) as usize
    }

    fn drain(&mut self, scratch: &mut Vec<u8>, consume: &mut dyn FnMut(&[u8])) {
        let _ = crate::runtime::host_output::drain(
            &mut self.bus,
            &mut self.host_interface,
            scratch,
            |_, _, bytes| {
                consume(bytes);
                Ok(())
            },
        );
    }

    fn inspect(
        &self,
        address: u64,
        data: &mut [u8],
    ) -> Result<(), EmulatorError> {
        super::super::inspect_memory(
            &self.bus,
            |location| A::physical(&self.processor, &self.bus, location),
            address,
            data,
        )
    }

    fn monitor(&self, words: &[&str]) -> Option<Result<String, &'static str>> {
        A::monitor(self, words)
    }

    fn pause_host_clock(&mut self) {
        Self::pause_host_clock(self);
    }

    fn program_counter(&self) -> u64 {
        self.processor.program_counter().to_u64()
    }

    fn registers(&self) -> Vec<Described> {
        A::described_registers(&self.processor)
    }

    fn return_address(&self) -> Option<u64> {
        A::return_address(&self.processor)
    }

    fn thread(&self) -> &'static str {
        A::THREAD
    }
}

pub trait System {
    fn advance(&mut self) -> Result<Advanced, EmulatorError>;

    fn digits(&self) -> usize;

    fn drain(&mut self, scratch: &mut Vec<u8>, consume: &mut dyn FnMut(&[u8]));

    fn inspect(
        &self,
        address: u64,
        data: &mut [u8],
    ) -> Result<(), EmulatorError>;

    fn monitor(&self, words: &[&str]) -> Option<Result<String, &'static str>>;

    fn pause_host_clock(&mut self);

    fn program_counter(&self) -> u64;

    fn registers(&self) -> Vec<Described>;

    fn return_address(&self) -> Option<u64>;

    fn thread(&self) -> &'static str;
}
