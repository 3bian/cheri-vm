// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::Instruction;

pub fn machine(word: u32) -> Option<Instruction> {
    match system(word)? {
        0x6040 => Some(Instruction::MachineReturn),
        0x20a0 => Some(Instruction::WaitForInterrupt),
        _ => None,
    }
}

pub fn supervisor(word: u32) -> Option<Instruction> {
    match system(word)? {
        0x2040 => Some(Instruction::SupervisorReturn),
        operands if operands >> 10 == 0x09 => {
            Some(Instruction::SupervisorFence)
        }
        _ => None,
    }
}

fn system(word: u32) -> Option<u32> {
    (word & 0x7f == 0x73 && (word >> 12) & 7 == 0 && (word >> 7) & 31 == 0)
        .then_some(word >> 15)
}
