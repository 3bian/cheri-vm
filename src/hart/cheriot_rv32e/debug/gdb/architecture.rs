// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use gdbstub::arch::Arch;

use super::{RegisterId, Registers};

pub(crate) enum Architecture {}

impl Arch for Architecture {
    type BreakpointKind = usize;
    type RegId = RegisterId;
    type Registers = Registers;
    type Usize = u32;

    fn target_description_xml() -> Option<&'static str> {
        Some(include_str!("target.xml"))
    }
}
