// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

#[derive(Clone, Copy)]
pub(super) struct Fields {
    pub destination: u8,
    pub first: u8,
    pub function_code: u32,
    pub opcode: u32,
    pub prefix: u32,
    pub second: u8,
    pub selector: u32,
    pub wide: bool,
    pub word: u32,
}

impl Fields {
    pub(super) fn immediate(
        self,
        operation: ImmediateOperation,
        immediate: i32,
    ) -> Instruction {
        Instruction::Immediate {
            operation,
            destination: self.destination,
            source: self.first,
            immediate,
        }
    }

    pub(super) fn immediate_word(
        self,
        operation: ImmediateOperation,
        immediate: i32,
    ) -> Instruction {
        Instruction::ImmediateWord {
            operation,
            destination: self.destination,
            source: self.first,
            immediate,
        }
    }

    pub(super) fn of(word: u32, xlen: u32) -> Self {
        Self {
            destination: ((word >> 7) & 31) as u8,
            first: ((word >> 15) & 31) as u8,
            function_code: (word >> 12) & 7,
            opcode: word & 0x7f,
            prefix: word >> 25,
            second: ((word >> 20) & 31) as u8,
            selector: word >> 20,
            wide: xlen == 64,
            word,
        }
    }

    pub(super) fn register(self, operation: RegisterOperation) -> Instruction {
        Instruction::Register {
            operation,
            destination: self.destination,
            first: self.first,
            second: self.second,
        }
    }

    pub(super) fn register_word(
        self,
        operation: RegisterOperation,
    ) -> Instruction {
        Instruction::RegisterWord {
            operation,
            destination: self.destination,
            first: self.first,
            second: self.second,
        }
    }

    pub(super) fn shift(self) -> Option<(u32, i32)> {
        if self.wide {
            Some((self.word >> 26, ((self.word >> 20) & 63) as i32))
        } else if self.word & (1 << 25) == 0 {
            Some((self.word >> 26, i32::from(self.second)))
        } else {
            None
        }
    }
}
