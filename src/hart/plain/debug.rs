// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[cfg(feature = "gdb")]
pub(crate) mod gdb;
mod model;

pub(crate) use model::DebugModel;

use crate::debug::architecture::{DebugArchitecture, Described};
use crate::hart::Hart;
use crate::hart::plain::{Plain, Processor};
use crate::hart::privilege::PrivilegeModel;
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::bus::Bus;

const REGISTER_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1",
    "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

impl<X: Xlen, P: PrivilegeModel, M: DebugModel<X>> DebugArchitecture
    for Plain<X, P, M>
{
    const THREAD: &'static str = "RISC-V core";

    fn described_registers(processor: &Processor<X, P, M>) -> Vec<Described> {
        let digits = (X::BITS / 4) as usize + 2;
        let mut registers = vec![Described {
            address: None,
            name: "pc".into(),
            value: M::describe_program_counter(
                &processor.shadow,
                processor.program_counter,
            ),
        }];
        for (index, value) in processor.registers().iter().enumerate() {
            registers.push(Described {
                address: Some(value.to_u64()),
                name: REGISTER_NAMES[index].into(),
                value: M::describe_register(
                    &processor.shadow,
                    index as u8,
                    *value,
                ),
            });
        }
        if processor.extensions.contains(ExtensionSet::F) {
            for (index, bits) in processor.float_registers.iter().enumerate() {
                registers.push(Described {
                    address: None,
                    name: format!("f{index}"),
                    value: describe_float(processor.extensions, *bits),
                });
            }
            registers.push(Described {
                address: None,
                name: "fcsr".into(),
                value: format!("{:#0digits$x}", processor.float_control),
            });
        }
        registers
    }

    fn monitor(
        hart: &Hart<Self>,
        words: &[&str],
    ) -> Option<Result<String, &'static str>> {
        M::monitor(hart, words)
    }

    fn physical(
        processor: &Processor<X, P, M>,
        bus: &Bus,
        address: u64,
    ) -> Option<u64> {
        processor.privilege.inspect_address(
            &processor.machine_registers,
            bus,
            address,
        )
    }

    fn return_address(processor: &Processor<X, P, M>) -> Option<u64> {
        M::return_address(
            &processor.shadow,
            processor.read_register_unchecked(1),
        )
    }
}

pub(crate) fn describe_float(extensions: ExtensionSet, bits: u64) -> String {
    if extensions.contains(ExtensionSet::D) && bits >> 32 != 0xffff_ffff {
        format!("{bits:#018x} ({:?})", f64::from_bits(bits))
    } else {
        format!("{bits:#018x} ({:?}, single)", f32::from_bits(bits as u32))
    }
}
