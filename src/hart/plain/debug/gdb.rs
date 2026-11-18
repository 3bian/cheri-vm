// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod architecture;
mod register_id;
mod registers;

pub(crate) use architecture::Width;
pub(crate) use register_id::RegisterId;
pub(crate) use registers::Registers;

use super::DebugModel;
use crate::debug::gdb::GdbArchitecture;
use crate::hart::plain::model::rvy::Shadow;
use crate::hart::plain::{Flat, Plain, Processor};
use crate::hart::privilege::PrivilegeModel;
use crate::isa::extension::ExtensionSet;
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::Capability128;
use crate::isa::xlen::{Rv64, Xlen};

impl<X: Xlen> GdbModel<X> for Flat {
    type Context = ();

    const MONITOR_HELP: &'static str = "help | fault | status\nInteger \
        registers and CSRs are read-only. Unimplemented E registers and \
        disabled CSRs are unavailable.\n";

    fn context((): &(), _: X::Word) {}

    fn describe_context((): &()) -> String {
        String::new()
    }
}

pub(crate) trait GdbModel<X: Xlen>: DebugModel<X> {
    type Context: Copy;

    const MONITOR_HELP: &'static str;

    fn context(
        shadow: &Self::Shadow,
        program_counter: X::Word,
    ) -> Self::Context;

    fn describe_context(context: &Self::Context) -> String;
}

impl<X: Width, P: PrivilegeModel, M: GdbModel<X>> GdbArchitecture
    for Plain<X, P, M>
{
    const ANY_BREAKPOINT_ADDRESS: bool = P::SUPERVISOR;
    type Arch = X::Arch;
    type Context = M::Context;
    const MONITOR_HELP: &'static str = M::MONITOR_HELP;

    fn context(processor: &Processor<X, P, M>) -> M::Context {
        M::context(&processor.shadow, processor.program_counter)
    }

    fn describe_context(context: &M::Context) -> String {
        M::describe_context(context)
    }

    fn describe_processor(processor: &Processor<X, P, M>) -> String {
        let mode = if P::SUPERVISOR {
            format!(" mode={:?}", processor.privilege.mode())
        } else {
            String::new()
        };
        format!(
            " registers={} compressed={}{mode}",
            processor.extensions.register_count(),
            processor.extensions.contains(ExtensionSet::C)
        )
    }

    fn read_register(
        processor: &Processor<X, P, M>,
        id: RegisterId<X>,
        bytes: &mut [u8],
    ) -> Option<usize> {
        Registers::new(processor).register(id, bytes)
    }

    fn registers(processor: &Processor<X, P, M>) -> Registers<X> {
        Registers::new(processor)
    }

    fn target_description(processor: &Processor<X, P, M>) -> Option<String> {
        (X::BITS == 64).then(|| Registers::new(processor).description())
    }
}

impl GdbModel<Rv64> for Rvy {
    type Context = Capability128;

    const MONITOR_HELP: &'static str = "help | capability \
         x0..x31|pcc|mtvec|mepc|mscratch|mtidc|stvec|sepc|sscratch|stidc|utidc \
         | tags ADDRESS COUNT | fault | status\nRegisters are read-only. A \
         capability's address is the integer register GDB shows. Tags are \
         stored tags, in aligned sixteen-byte granules at physical \
         addresses.\n";

    fn context(shadow: &Shadow, program_counter: u64) -> Capability128 {
        shadow.program_counter().at(program_counter)
    }

    fn describe_context(previous: &Capability128) -> String {
        format!(
            " pre_pcc=0x{:016x}/{}",
            previous.metadata,
            u8::from(previous.tag)
        )
    }
}
