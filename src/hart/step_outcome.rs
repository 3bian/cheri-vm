// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::DecodedInstruction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepOutcome<W = u32> {
    Retired {
        address: W,
        decoded: DecodedInstruction,
    },
    Trapped {
        address: W,
        cause: u32,
        interrupt: bool,
        value: W,
    },
}
