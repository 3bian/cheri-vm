// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::{RegisterId, Registers};
use super::Architecture;
use crate::isa::xlen::{Rv32, Rv64, Xlen};
use gdbstub::arch::Arch;

impl Width for Rv32 {
    type Arch = Architecture<Self>;
}

impl Width for Rv64 {
    type Arch = Architecture<Self>;
}

pub(crate) trait Width: Xlen + Copy + core::fmt::Debug + Eq {
    type Arch: Arch<
            BreakpointKind = usize,
            RegId = RegisterId<Self>,
            Registers = Registers<Self>,
            Usize: Into<u64>,
        >;
}
