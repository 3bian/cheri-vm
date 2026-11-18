// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod width;

pub(crate) use width::Width;

use core::marker::PhantomData;

use gdbstub::arch::Arch;

use super::{RegisterId, Registers};

use crate::isa::xlen::{Rv32, Rv64};

pub(crate) struct Architecture<X>(PhantomData<X>);

impl Arch for Architecture<Rv32> {
    type BreakpointKind = usize;
    type RegId = RegisterId<Rv32>;
    type Registers = Registers<Rv32>;
    type Usize = u32;

    fn target_description_xml() -> Option<&'static str> {
        Some(include_str!("target.xml"))
    }
}

impl Arch for Architecture<Rv64> {
    type BreakpointKind = usize;
    type RegId = RegisterId<Rv64>;
    type Registers = Registers<Rv64>;
    type Usize = u64;
}
