// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::ProcessorState;
use crate::hart::machine_registers::{FLOAT_STATE, MachineRegisters};
use crate::hart::plain::model::{FetchOf, Flat, Model};
use crate::hart::privilege::{MachineOnly, PrivilegeModel};
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::{Word, Xlen};

pub struct Processor<X: Xlen, P = MachineOnly, M: Model<X> = Flat> {
    pub extensions: ExtensionSet,
    pub float_control: u32,
    pub float_registers: [u64; 32],
    pub machine_registers: MachineRegisters<X::Word>,
    pub privilege: P,
    pub program_counter: X::Word,
    registers: [X::Word; 32],
    pub(crate) reservation: Option<u64>,
    pub(crate) shadow: M::Shadow,
}

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Processor<X, P, M> {
    pub(crate) fn float_csr(&self, address: u16) -> Option<u32> {
        if !self.float_enabled() {
            return None;
        }
        Some(match address {
            1 => self.float_control & 0x1f,
            2 => (self.float_control >> 5) & 7,
            _ => self.float_control,
        })
    }

    pub(crate) fn float_enabled(&self) -> bool {
        self.machine_registers.machine_status.to_u64() & FLOAT_STATE != 0
    }

    pub(crate) fn new(extensions: ExtensionSet, entry: X::Word) -> Self {
        let letter = |extension, bit: u32| {
            u64::from(extensions.contains(extension)) << bit
        };
        let mut machine_registers = MachineRegisters {
            instruction_alignment: extensions.instruction_alignment(),
            isa: X::Word::from_u64(
                (u64::from(X::BITS / 32) << (X::BITS - 2))
                    | letter(ExtensionSet::A, 0)
                    | letter(
                        ExtensionSet::ZBA
                            .with(ExtensionSet::ZBB)
                            .with(ExtensionSet::ZBS),
                        1,
                    )
                    | letter(ExtensionSet::C, 2)
                    | letter(ExtensionSet::D, 3)
                    | letter(ExtensionSet::F, 5)
                    | letter(ExtensionSet::E, 4)
                    | letter(ExtensionSet::I, 8)
                    | letter(ExtensionSet::M, 12)
                    | letter(ExtensionSet::S, 18)
                    | letter(ExtensionSet::U, 20)
                    | letter(ExtensionSet::Y, 24),
            ),
            ..MachineRegisters::default()
        };
        let privilege = P::new(extensions, &mut machine_registers);
        Self {
            extensions,
            float_control: 0,
            float_registers: [0; 32],
            machine_registers,
            privilege,
            program_counter: entry,
            registers: [X::Word::default(); 32],
            reservation: None,
            shadow: M::shadow(extensions),
        }
    }

    pub(crate) fn read_register_unchecked(&self, index: u8) -> X::Word {
        if index == 0 {
            X::Word::default()
        } else {
            self.registers[index as usize]
        }
    }

    pub(crate) fn registers(&self) -> &[X::Word] {
        &self.registers[..self.extensions.register_count()]
    }

    pub(crate) fn set_float_csr(&mut self, address: u16, value: u32) {
        self.float_control = match address {
            1 => (self.float_control & !0x1f) | (value & 0x1f),
            2 => (self.float_control & 0x1f) | ((value & 7) << 5),
            _ => value & 0xff,
        };
        self.touch_float();
    }

    pub(crate) fn touch_float(&mut self) {
        let status = self.machine_registers.machine_status.to_u64();
        self.machine_registers.machine_status =
            X::Word::from_u64(status | FLOAT_STATE);
    }

    pub(crate) fn validate_register(&self, index: u8) -> bool {
        (index as usize) < self.extensions.register_count()
    }

    pub(crate) fn write_address(&mut self, index: u8, value: X::Word) {
        if index != 0 {
            self.registers[index as usize] = value;
        }
    }

    pub(crate) fn write_register_unchecked(
        &mut self,
        index: u8,
        value: X::Word,
    ) {
        if index != 0 {
            self.registers[index as usize] = value;
            M::clear(&mut self.shadow, index);
        }
    }
}

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> ProcessorState<M::Pointer>
    for Processor<X, P, M>
{
    fn fetch_authority(&self) -> FetchOf<X, M> {
        M::authority(&self.shadow, self.program_counter)
    }

    fn machine_registers(&self) -> &MachineRegisters<X::Word> {
        &self.machine_registers
    }

    fn machine_registers_mut(&mut self) -> &mut MachineRegisters<X::Word> {
        &mut self.machine_registers
    }

    fn program_counter(&self) -> X::Word {
        self.program_counter
    }

    fn stack_pointer(&self) -> X::Word {
        self.registers[2]
    }
}
