// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[cfg(feature = "gdb")]
pub(crate) mod gdb;
mod monitor;

use crate::debug::architecture::{DebugArchitecture, Described};
use crate::hart::Hart;
use crate::hart::cheriot_rv32e::{CheriotRv32e, Processor};

const REGISTER_NAMES: [&str; 16] = [
    "czero", "cra", "csp", "cgp", "ctp", "ct0", "ct1", "ct2", "cs0", "cs1",
    "ca0", "ca1", "ca2", "ca3", "ca4", "ca5",
];

impl DebugArchitecture for CheriotRv32e {
    const THREAD: &'static str = "CHERIoT core";

    fn described_registers(processor: &Processor) -> Vec<Described> {
        let mut registers = vec![Described {
            address: None,
            name: "pc".into(),
            value: format!("{:#010x}", processor.program_counter),
        }];
        for (name, capability) in
            REGISTER_NAMES.iter().zip(processor.registers())
        {
            let (base, top) = capability.bounds();
            registers.push(Described {
                address: Some(capability.address().into()),
                name: (*name).into(),
                value: format!(
                    "{:#010x} (tag={} base={base:#010x} top={top:#x} \
                     perms={:#05x}{})",
                    capability.address(),
                    u8::from(capability.tag()),
                    capability.permissions(),
                    if capability.sealed() { " sealed" } else { "" },
                ),
            });
        }
        registers
    }

    fn monitor(
        hart: &Hart<Self>,
        words: &[&str],
    ) -> Option<Result<String, &'static str>> {
        match words {
            ["capability", name] => {
                Some(monitor::describe_capability(&hart.processor, name))
            }
            ["tags", address, count] => {
                Some(monitor::tags(&hart.bus, address, count))
            }
            _ => None,
        }
    }

    fn return_address(processor: &Processor) -> Option<u64> {
        let link = processor.registers()[1];
        link.tag().then(|| link.address().into())
    }
}
