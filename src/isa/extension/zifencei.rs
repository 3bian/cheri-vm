// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::Instruction;

pub fn standard(word: u32, _: u32) -> Option<Instruction> {
    (word & 0x7f == 0x0f && (word >> 12) & 7 == 1)
        .then_some(Instruction::FenceInstruction)
}
