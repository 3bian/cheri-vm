// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod slot;

use slot::Slot;

use std::fmt::Write;

use gdbstub::arch::Registers as GdbRegisters;

use super::register_id::{CSR_ADDRESSES, FIRST_CSR, FIRST_FLOAT, PRIVILEGE};

use super::{RegisterId, Width};

use crate::hart::plain::Processor;

use crate::hart::plain::debug::DebugModel;

use crate::hart::privilege::PrivilegeModel;

use crate::isa::extension::ExtensionSet;

use crate::isa::xlen::Word;

const SUPERVISOR_CSRS: [(u16, &str); 13] = [
    (0x100, "sstatus"),
    (0x104, "sie"),
    (0x105, "stvec"),
    (0x106, "scounteren"),
    (0x140, "sscratch"),
    (0x141, "sepc"),
    (0x142, "scause"),
    (0x143, "stval"),
    (0x144, "sip"),
    (0x180, "satp"),
    (0x302, "medeleg"),
    (0x303, "mideleg"),
    (0x306, "mcounteren"),
];

const WIDE_CSRS: [(u16, &str); 19] = [
    (0x300, "mstatus"),
    (0x301, "misa"),
    (0x304, "mie"),
    (0x305, "mtvec"),
    (0x320, "mcountinhibit"),
    (0x340, "mscratch"),
    (0x341, "mepc"),
    (0x342, "mcause"),
    (0x343, "mtval"),
    (0x344, "mip"),
    (0xb00, "mcycle"),
    (0xb02, "minstret"),
    (0xc00, "cycle"),
    (0xc02, "instret"),
    (0xf11, "mvendorid"),
    (0xf12, "marchid"),
    (0xf13, "mimpid"),
    (0xf14, "mhartid"),
    (0xf15, "mconfigptr"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Registers<X: Width> {
    pc: X::Word,
    slots: Vec<Slot>,
}

impl<X: Width> Registers<X> {
    pub(crate) fn description(&self) -> String {
        let mut text = format!(
            "<?xml version=\"1.0\"?>\n<!DOCTYPE target SYSTEM \
             \"gdb-target.dtd\">\n<target version=\"1.0\">\n  \
             <architecture>riscv:rv{}</architecture>\n",
            X::BITS
        );
        let mut feature = "";
        for slot in &self.slots {
            let (next, name, kind) = match slot.raw {
                0..32 => ("cpu", format!("x{}", slot.raw), "int"),
                32 => ("cpu", "pc".into(), "code_ptr"),
                FIRST_FLOAT..=64 => (
                    "fpu",
                    format!("f{}", slot.raw - FIRST_FLOAT),
                    if slot.bits == 64 {
                        "ieee_double"
                    } else {
                        "ieee_single"
                    },
                ),
                66..=68 => ("fpu", slot.name.into(), "int"),
                PRIVILEGE => ("virtual", slot.name.into(), "int"),
                _ => ("csr", slot.name.into(), "int"),
            };
            if next != feature {
                if !feature.is_empty() {
                    text.push_str("  </feature>\n");
                }
                let _ = writeln!(
                    text,
                    "  <feature name=\"org.gnu.gdb.riscv.{next}\">"
                );
                feature = next;
            }
            let _ = writeln!(
                text,
                "    <reg name=\"{name}\" bitsize=\"{}\" regnum=\"{}\" \
                 type=\"{kind}\"/>",
                slot.bits, slot.raw
            );
        }
        text.push_str("  </feature>\n</target>\n");
        text
    }

    pub(crate) fn new<P: PrivilegeModel, M: DebugModel<X>>(
        processor: &Processor<X, P, M>,
    ) -> Self {
        let word = X::BITS as usize;
        let mut slots = Vec::with_capacity(96);
        let held = processor.registers();
        for index in 0..32 {
            slots.push(Slot {
                bits: word,
                name: "",
                raw: index,
                value: held.get(index).map(|value| value.to_u64()),
            });
        }
        slots.push(Slot {
            bits: word,
            name: "pc",
            raw: 32,
            value: Some(processor.program_counter.to_u64()),
        });
        let csr = |address: u16| {
            processor
                .extensions
                .contains(ExtensionSet::ZICSR)
                .then(|| {
                    processor.privilege.inspect_csr::<X::Word>(
                        &processor.machine_registers,
                        address,
                    )
                })
                .flatten()
                .map(Word::to_u64)
        };
        if X::BITS == 32 {
            slots.extend(CSR_ADDRESSES.iter().map(|address| Slot {
                bits: word,
                name: "",
                raw: FIRST_CSR + usize::from(*address),
                value: csr(*address),
            }));
            return Self {
                pc: processor.program_counter,
                slots,
            };
        }
        if processor.extensions.contains(ExtensionSet::F) {
            let double = processor.extensions.contains(ExtensionSet::D);
            for (index, value) in processor.float_registers.iter().enumerate() {
                slots.push(Slot {
                    bits: if double { 64 } else { 32 },
                    name: "",
                    raw: FIRST_FLOAT + index,
                    value: Some(if double {
                        *value
                    } else {
                        *value & 0xffff_ffff
                    }),
                });
            }
            let control = u64::from(processor.float_control);
            for (address, name, value) in [
                (1, "fflags", control & 0x1f),
                (2, "frm", (control >> 5) & 7),
                (3, "fcsr", control),
            ] {
                slots.push(Slot {
                    bits: 32,
                    name,
                    raw: FIRST_CSR + address,
                    value: Some(value),
                });
            }
        }
        let supervisor: &[(u16, &str)] =
            if P::SUPERVISOR { &SUPERVISOR_CSRS } else { &[] };
        let mut listed: Vec<_> = WIDE_CSRS.iter().chain(supervisor).collect();
        listed.sort();
        slots.extend(listed.into_iter().map(|(address, name)| Slot {
            bits: word,
            name,
            raw: FIRST_CSR + usize::from(*address),
            value: csr(*address),
        }));
        if P::SUPERVISOR {
            slots.push(Slot {
                bits: word,
                name: "priv",
                raw: PRIVILEGE,
                value: Some(processor.privilege.mode() as u64),
            });
        }
        Self {
            pc: processor.program_counter,
            slots,
        }
    }

    pub(crate) fn register(
        &self,
        id: RegisterId<X>,
        output: &mut [u8],
    ) -> Option<usize> {
        let slot = self.slots.iter().find(|slot| slot.raw == id.raw())?;
        let bytes = slot.bits / 8;
        if output.len() < bytes {
            return None;
        }
        let Some(value) = slot.value else {
            return Some(0);
        };
        output[..bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
        Some(bytes)
    }
}

impl<X: Width> Default for Registers<X> {
    fn default() -> Self {
        Self {
            pc: X::Word::default(),
            slots: Vec::new(),
        }
    }
}

impl<X: Width> GdbRegisters for Registers<X> {
    type ProgramCounter = X::Word;

    fn gdb_deserialize(&mut self, _: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        for slot in &self.slots {
            let bytes = slot.value.map(u64::to_le_bytes);
            for index in 0..slot.bits / 8 {
                write_byte(bytes.map(|bytes| bytes[index]));
            }
        }
    }

    fn pc(&self) -> X::Word {
        self.pc
    }
}
