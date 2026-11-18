// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::machine_registers::MachineRegisters;
use crate::hart::plain::Processor;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::{Rv32, Xlen};
use crate::machine::EmulatorError;

impl<X: Xlen, P: PrivilegeModel> Processor<X, P> {
    pub(crate) fn read_register(
        &self,
        index: u8,
    ) -> Result<X::Word, EmulatorError> {
        self.registers()
            .get(usize::from(index))
            .copied()
            .ok_or_else(|| self.register_error(index))
    }

    fn register_error(&self, index: u8) -> EmulatorError {
        EmulatorError::Configuration(format!(
            "register x{index} is unavailable on this {}-register processor",
            self.extensions.register_count()
        ))
    }

    pub(crate) fn write_register(
        &mut self,
        index: u8,
        value: X::Word,
    ) -> Result<(), EmulatorError> {
        if !self.validate_register(index) {
            return Err(self.register_error(index));
        }

        self.write_register_unchecked(index, value);
        Ok(())
    }
}

#[test]
fn a_trap_saves_the_machine_state_and_mret_restores_it() {
    let mut registers = MachineRegisters::<u32> {
        trap_vector: 0x101,
        machine_status: 1 << 3,
        ..Default::default()
    };
    assert_eq!(registers.enter_trap(0x200, 7, 0, true), 0x11c);
    assert_eq!(registers.exception_program_counter, 0x200);
    assert_eq!(registers.trap_cause, (1 << 31) | 7);
    assert_eq!(registers.machine_return(), 0x200);
    assert_ne!(registers.machine_status & (1 << 3), 0);
}

#[test]
fn at_64_bits_the_interrupt_is_the_top_bit_and_a_counter_is_one_register() {
    let mut registers = MachineRegisters::<u64> {
        trap_vector: 0x1_0000_0001,
        machine_status: 1 << 3,
        cycle: 0x1_2345_6789,
        ..Default::default()
    };
    assert_eq!(
        registers.enter_trap(0x2_0000_0000, 7, u64::MAX, true),
        0x1_0000_001c
    );
    assert_eq!(registers.trap_cause, (1 << 63) | 7);
    assert_eq!(registers.read(0x343), Some(u64::MAX));
    assert_eq!(registers.machine_return(), 0x2_0000_0000);
    assert_eq!(registers.read(0xb00), Some(0x1_2345_6789));
    assert_eq!(registers.read(0xb80), None);
    assert_eq!(registers.read(0x310), None);
    assert!(registers.write(0xb80, 1).is_err());
    registers.write(0xb00, 0xffff_ffff_0000_0001).unwrap();
    assert_eq!(registers.cycle, 0xffff_ffff_0000_0001);

    let mut narrow = MachineRegisters::<u32> {
        cycle: 0x1_2345_6789,
        ..Default::default()
    };
    assert_eq!(narrow.read(0xb00), Some(0x2345_6789));
    assert_eq!(narrow.read(0xb80), Some(1));
    assert_eq!(narrow.read(0x310), Some(0));
    narrow.write(0xb80, 7).unwrap();
    assert_eq!(narrow.cycle, 0x7_2345_6789);
}

#[test]
fn the_isa_register_names_the_word_size_and_the_extensions() {
    use crate::isa::xlen::Rv64;
    let set = ExtensionSet::I
        .with(ExtensionSet::M)
        .with(ExtensionSet::A)
        .with(ExtensionSet::C);
    assert_eq!(
        Processor::<Rv64>::new(set, 0).machine_registers.isa,
        0x8000_0000_0000_1105
    );
    assert_eq!(
        Processor::<Rv32>::new(set, 0).machine_registers.isa,
        0x4000_1105
    );
    assert_eq!(
        Processor::<Rv32>::new(ExtensionSet::E, 0)
            .machine_registers
            .isa,
        0x4000_0010
    );
}

#[test]
fn zero_register_remains_zero() {
    let profile = ExtensionSet::I.with(ExtensionSet::ZICSR);
    let mut processor = Processor::<Rv32>::new(profile, 0);
    processor.write_register_unchecked(0, 23);
    assert_eq!(processor.read_register_unchecked(0), 0);
}
