// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod architecture;
mod register_id;
mod registers;

pub(crate) use architecture::Architecture;
pub(crate) use register_id::RegisterId;
pub(crate) use registers::Registers;

use crate::debug::gdb::GdbArchitecture;
use crate::hart::cheriot_rv32e::{CheriotRv32e, Processor};
use crate::isa::pointer::cheriot::Capability;

impl GdbArchitecture for CheriotRv32e {
    const ANY_BREAKPOINT_ADDRESS: bool = false;
    type Arch = Architecture;
    type Context = Capability;

    const MONITOR_HELP: &'static str = "help | capability \
        c0..c15|pcc|mtcc|mtdc|mscratchc|mepcc | tags ADDRESS COUNT | fault | \
        status\nTags are stored tags, in aligned eight-byte granules; no \
        revocation filtering.\n";

    fn context(processor: &Processor) -> Capability {
        processor.pcc
    }

    fn describe_context(previous_pcc: &Capability) -> String {
        let (payload, tag) = previous_pcc.to_memory();
        format!(" pre_pcc=0x{payload:016x}/{}", u8::from(tag))
    }

    fn describe_processor(_: &Processor) -> String {
        String::new()
    }

    fn read_register(
        processor: &Processor,
        id: RegisterId,
        bytes: &mut [u8],
    ) -> Option<usize> {
        Registers::new(processor).register(id, bytes)
    }

    fn registers(processor: &Processor) -> Registers {
        Registers::new(processor)
    }
}
