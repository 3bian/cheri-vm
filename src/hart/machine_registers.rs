// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::xlen::Word;
use crate::machine::error::EmulatorError;

pub(crate) const FLOAT_STATE: u64 = 3 << 13;

#[derive(Default, Clone, Debug)]
pub struct MachineRegisters<W = u32> {
    pub counter_inhibit: u32,
    pub cycle: u64,
    pub exception_program_counter: W,
    pub instruction_alignment: u32,
    pub interrupt_enable: u32,
    pub interrupt_pending: u32,
    pub isa: W,
    pub machine_status: W,
    pub retired: u64,
    pub scratch: W,
    pub time: u64,
    pub trap_cause: W,
    pub trap_value: W,
    pub trap_vector: W,
}

impl<W: Word> MachineRegisters<W> {
    pub(crate) fn enter_trap(
        &mut self,
        address: W,
        cause: u32,
        value: W,
        interrupt: bool,
    ) -> W {
        self.exception_program_counter = address;
        self.trap_cause = W::from_u64(
            u64::from(cause) | (u64::from(interrupt) << (W::BITS - 1)),
        );
        self.trap_value = value;
        let status = self.machine_status.to_u64();
        self.machine_status = W::from_u64(
            (status & !(1 << 3 | 1 << 7))
                | (status & (1 << 3)) << 4
                | (3 << 11),
        );
        let vector = self.trap_vector.to_u64();
        let base = vector & !3;

        W::from_u64(if interrupt && vector & 3 == 1 {
            base.wrapping_add(4 * u64::from(cause & 0x7fff_ffff))
        } else {
            base
        })
    }

    pub(crate) fn exception_address(&self) -> W {
        W::from_u64(
            self.exception_program_counter.to_u64()
                & !(u64::from(self.instruction_alignment.max(2)) - 1),
        )
    }

    pub(crate) fn float_state(&self) -> u64 {
        if self.isa.to_u64() & (1 << 5) != 0 {
            FLOAT_STATE
        } else {
            0
        }
    }

    pub(crate) fn machine_return(&mut self) -> W {
        let status = self.machine_status.to_u64();
        self.machine_status = W::from_u64(
            (status & !(1 << 3))
                | (status & (1 << 7)) >> 4
                | (1 << 7)
                | (3 << 11),
        );
        self.exception_address()
    }

    pub(crate) fn read(&self, address: u16) -> Option<W> {
        let narrow = W::BITS == 32;
        let word = |value: u32| Some(W::from_u64(u64::from(value)));
        let counter = |value: u64| Some(W::from_u64(value));
        let high = |value: u64| narrow.then(|| W::from_u64(value >> 32));
        match address {
            0x300 => Some(W::from_u64(with_summary(
                self.machine_status.to_u64(),
                W::BITS,
            ))),
            0x301 => Some(self.isa),
            0x310 if narrow => word(0),
            0xf11..=0xf15 => word(0),
            0x320 => word(self.counter_inhibit),
            0x304 => word(self.interrupt_enable),
            0x305 => Some(self.trap_vector),
            0x340 => Some(self.scratch),
            0x341 => Some(self.exception_address()),
            0x342 => Some(self.trap_cause),
            0x343 => Some(self.trap_value),
            0x344 => word(self.interrupt_pending),
            0xb00 | 0xc00 => counter(self.cycle),
            0xb80 | 0xc80 => high(self.cycle),
            0xc01 => counter(self.time),
            0xc81 => high(self.time),
            0xb02 | 0xc02 => counter(self.retired),
            0xb82 | 0xc82 => high(self.retired),
            _ => None,
        }
    }

    pub(crate) fn write(
        &mut self,
        address: u16,
        value: W,
    ) -> Result<(), EmulatorError> {
        let error = || {
            EmulatorError::Refused(format!(
                "CSR 0x{address:03x} is not writable or implemented"
            ))
        };
        let narrow = W::BITS == 32;
        let wide = value.to_u64();
        let low = wide as u32;
        let counter = |current: u64| {
            if narrow {
                (current & !0xffff_ffff) | u64::from(low)
            } else {
                wide
            }
        };
        let high =
            |current: u64| (current & 0xffff_ffff) | (u64::from(low) << 32);

        match address {
            0x300 => {
                let kept = (1 << 3) | (1 << 7) | self.float_state();
                self.machine_status = W::from_u64(
                    (self.machine_status.to_u64() & !kept)
                        | (wide & kept)
                        | (3 << 11),
                )
            }
            0x304 => {
                self.interrupt_enable = low & ((1 << 3) | (1 << 7) | (1 << 11))
            }

            0x305 => {
                self.trap_vector =
                    W::from_u64((wide & !3) | u64::from(wide & 3 == 1))
            }
            0x301 | 0x344 => {}
            0x310 if narrow => {}
            0x320 => self.counter_inhibit = low & 5,
            0x340 => self.scratch = value,
            0x341 => self.exception_program_counter = W::from_u64(wide & !1),
            0x342 => self.trap_cause = value,
            0x343 => self.trap_value = value,
            0xb00 => self.cycle = counter(self.cycle),
            0xb80 if narrow => self.cycle = high(self.cycle),
            0xb02 => self.retired = counter(self.retired),
            0xb82 if narrow => self.retired = high(self.retired),
            _ => return Err(error()),
        }

        Ok(())
    }
}

pub(crate) fn with_summary(status: u64, bits: u32) -> u64 {
    if status & FLOAT_STATE == FLOAT_STATE {
        status | 1 << (bits - 1)
    } else {
        status
    }
}
