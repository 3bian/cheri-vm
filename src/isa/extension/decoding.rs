// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::instruction::Instruction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decoding {
    Absent,
    Found(Instruction),
    Reserved,
}

impl From<Option<Instruction>> for Decoding {
    fn from(instruction: Option<Instruction>) -> Self {
        instruction.map_or(Self::Absent, Self::Found)
    }
}
