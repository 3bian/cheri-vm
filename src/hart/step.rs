// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fault::INTERRUPT;
use super::{
    Architecture, Exception, Fault, Hart, ProcessorState, StepOutcome, WordOf,
};
use crate::hart::execution::TrapPolicy;
use crate::isa::decode::{self, DecodedInstruction};
use crate::isa::pointer::PointerModel;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::{EmulatorError, ExceptionCause};
use crate::machine::timing_policy::{TimingPolicy, synchronize_machine_time};

impl<A: Architecture> Hart<A> {
    #[inline]
    fn check_fetch(
        &mut self,
        address: WordOf<A>,
    ) -> Result<(), Fault<WordOf<A>>> {
        A::Pointer::check_fetch(self.processor.fetch_authority(), address)
            .map_err(|violation| A::fetch_fault(&mut self.processor, violation))
    }

    fn decode(
        &mut self,
        encoded: u32,
        decode: impl FnOnce(u32) -> Option<DecodedInstruction>,
    ) -> Result<DecodedInstruction, Fault<WordOf<A>>> {
        self.decode_cache
            .decode(encoded, decode)
            .ok_or_else(|| A::illegal_instruction(encoded))
    }

    fn fetch(
        &mut self,
        fetched: &mut bool,
        injected: Option<u32>,
    ) -> Result<DecodedInstruction, Fault<WordOf<A>>> {
        let address = self.processor.program_counter();
        self.check_fetch(address)?;
        let alignment =
            self.processor.machine_registers().instruction_alignment;
        if address.to_u64() & u64::from(alignment - 1) != 0 {
            return Err(Fault {
                cause: ExceptionCause::InstructionAddressMisaligned.code(),
                value: address,
            });
        }
        let first = self.fetch_halfword(
            address,
            injected.map(|encoded| encoded & 0xffff),
        )?;
        let extensions = A::extensions(&self.processor);
        if first & 3 != 3 {
            *fetched = true;
            return self.decode(first, |encoded| {
                decode::compressed(extensions, A::Xlen::BITS, encoded as u16)
            });
        }
        let second_address = address.add_signed(2);
        self.check_fetch(second_address)?;
        let second = self.fetch_halfword(
            second_address,
            injected.map(|encoded| encoded >> 16),
        )?;
        *fetched = true;
        self.decode(first | second << 16, |encoded| {
            decode::standard(extensions, A::Xlen::BITS, encoded)
        })
    }

    fn fetch_halfword(
        &mut self,
        address: WordOf<A>,
        injected: Option<u32>,
    ) -> Result<u32, Fault<WordOf<A>>> {
        match injected {
            Some(halfword) => Ok(halfword),
            None => {
                let physical = A::fetch_address(
                    &mut self.processor,
                    &mut self.bus,
                    address,
                )?;
                self.bus.fetch(physical, 2).map_err(|_| Fault {
                    cause: ExceptionCause::InstructionAccessFault.code(),
                    value: address,
                })
            }
        }
    }

    fn pending_interrupt(&mut self) -> Option<u32> {
        self.bus.refresh_interrupts();
        let lines = self.bus.machine_interrupt_bits();
        A::pending_interrupt(&mut self.processor, lines)
    }

    pub fn step(&mut self) -> Result<StepOutcome<WordOf<A>>, EmulatorError> {
        self.step_with(None)
    }

    pub(crate) fn step_with(
        &mut self,
        injected: Option<u32>,
    ) -> Result<StepOutcome<WordOf<A>>, EmulatorError> {
        if self.timing == TimingPolicy::Elapsed {
            self.host_clock.synchronize(
                &mut self.bus,
                self.processor.machine_registers_mut(),
            );
        } else {
            synchronize_machine_time(
                &mut self.bus,
                self.processor.machine_registers_mut(),
            );
            self.host_clock.reset();
        }
        A::begin_step(self);
        let address = self.processor.program_counter();
        let inhibit = self.processor.machine_registers().counter_inhibit;
        let mut fetched = false;
        self.attempted_steps = self.attempted_steps.wrapping_add(1);
        self.last_store = None;

        let result = match self.pending_interrupt() {
            Some(cause) => Err(Exception::Fault(Fault {
                cause: INTERRUPT | cause,
                value: WordOf::<A>::default(),
            })),
            None => self
                .fetch(&mut fetched, injected)
                .map_err(Exception::Fault)
                .and_then(|decoded| {
                    A::execute(self, decoded).map(|()| decoded)
                }),
        };
        let written_csr = result
            .as_ref()
            .ok()
            .and_then(|decoded| decoded.instruction.csr_write_address());
        let outcome = match result {
            Ok(decoded) => {
                if inhibit & 4 == 0
                    && !matches!(written_csr, Some(0xb02 | 0xb82))
                {
                    let registers = self.processor.machine_registers_mut();
                    registers.retired = registers.retired.wrapping_add(1);
                }
                Ok(StepOutcome::Retired { address, decoded })
            }
            Err(Exception::Fault(fault)) => {
                A::enter_trap(&mut self.processor, address, fault);
                self.trapped(address, fault)
            }
            Err(Exception::Fatal(error)) => Err(error),
        };
        self.fetched_instructions =
            self.fetched_instructions.wrapping_add(u64::from(fetched));
        self.timing.advance(
            &mut self.bus,
            self.processor.machine_registers_mut(),
            &mut self.step_clock,
            inhibit,
            matches!(written_csr, Some(0xb00 | 0xb80)),
        );
        A::end_step(self);

        if let (Some(interface), Some((address, width))) =
            (&mut self.host_interface, self.last_store)
            && let Some(code) =
                interface.observe_store(&mut self.bus, address, width)?
        {
            self.completion = Some(code);
        }
        outcome
    }

    fn trapped(
        &self,
        address: WordOf<A>,
        fault: Fault<WordOf<A>>,
    ) -> Result<StepOutcome<WordOf<A>>, EmulatorError> {
        let interrupt = fault.cause & INTERRUPT != 0;
        let cause = fault.cause & !INTERRUPT;
        if self.trap_policy == TrapPolicy::Stop {
            let address = address.to_u64();
            let value = fault.value.to_u64();
            return Err(match ExceptionCause::from_code(cause) {
                _ if interrupt => EmulatorError::Interrupt { address, cause },
                Some(cause) => EmulatorError::Trap {
                    cause,
                    address,
                    value,
                },
                None => EmulatorError::CheriotTrap {
                    address,
                    cause,
                    value,
                },
            });
        }
        Ok(StepOutcome::Trapped {
            address,
            cause,
            interrupt,
            value: fault.value,
        })
    }
}
