// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::fmt::Write;

use crate::hart::Hart;
use crate::hart::plain::model::rvy::{Metadata, Shadow};
use crate::hart::plain::{Flat, Model, Plain};
use crate::hart::privilege::PrivilegeModel;
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::Capability128;
use crate::isa::xlen::{Rv64, Word, Xlen};
use crate::machine::bus::Bus;

pub(crate) trait DebugModel<X: Xlen>: Model<X> {
    fn describe_program_counter(
        shadow: &Self::Shadow,
        value: X::Word,
    ) -> String;

    fn describe_register(
        shadow: &Self::Shadow,
        index: u8,
        value: X::Word,
    ) -> String;

    fn monitor<P: PrivilegeModel>(
        hart: &Hart<Plain<X, P, Self>>,
        words: &[&str],
    ) -> Option<Result<String, &'static str>>;

    fn return_address(shadow: &Self::Shadow, link: X::Word) -> Option<u64>;
}

impl<X: Xlen> DebugModel<X> for Flat {
    fn describe_program_counter((): &(), value: X::Word) -> String {
        address::<X>(value)
    }

    fn describe_register((): &(), _: u8, value: X::Word) -> String {
        address::<X>(value)
    }

    fn monitor<P: PrivilegeModel>(
        _: &Hart<Plain<X, P, Self>>,
        _: &[&str],
    ) -> Option<Result<String, &'static str>> {
        None
    }

    fn return_address((): &(), link: X::Word) -> Option<u64> {
        Some(link.to_u64())
    }
}

impl DebugModel<Rv64> for Rvy {
    fn describe_program_counter(shadow: &Shadow, value: u64) -> String {
        describe_capability(shadow, shadow.program_counter().at(value))
    }

    fn describe_register(shadow: &Shadow, index: u8, value: u64) -> String {
        let held = if index == 0 {
            Capability128::default()
        } else {
            shadow.registers[usize::from(index)].at(value)
        };
        describe_capability(shadow, held)
    }

    fn monitor<P: PrivilegeModel>(
        hart: &Hart<Plain<Rv64, P, Self>>,
        words: &[&str],
    ) -> Option<Result<String, &'static str>> {
        match words {
            ["capability", name] => Some(
                capability(hart, name)
                    .map(|capability| {
                        format!(
                            "{name}: metadata=0x{:016x} {}\n",
                            capability.metadata,
                            describe_capability(
                                &hart.processor.shadow,
                                capability
                            )
                        )
                    })
                    .ok_or("unknown capability register"),
            ),
            ["tags", address, count] => Some(tags(&hart.bus, address, count)),
            _ => None,
        }
    }

    fn return_address(shadow: &Shadow, link: u64) -> Option<u64> {
        shadow.registers[1].tag.then_some(link)
    }
}

fn address<X: Xlen>(value: X::Word) -> String {
    let digits = (X::BITS / 4) as usize + 2;
    format!("{value:#0digits$x}")
}

fn capability<P: PrivilegeModel>(
    hart: &Hart<Plain<Rv64, P, Rvy>>,
    name: &str,
) -> Option<Capability128> {
    let processor = &hart.processor;
    let shadow = &processor.shadow;
    let widened = |address: u16, metadata: Metadata| {
        processor
            .privilege
            .inspect_csr::<u64>(&processor.machine_registers, address)
            .map(|value| metadata.at(value))
    };
    match name {
        "pcc" => Some(processor.program_counter_capability()),
        "mtvec" => widened(0x305, shadow.trap_vectors[0]),
        "mepc" => widened(0x341, shadow.exception_program_counters[0]),
        "mscratch" => widened(0x340, shadow.scratches[0]),
        "stvec" => widened(0x105, shadow.trap_vectors[1]),
        "sepc" => widened(0x141, shadow.exception_program_counters[1]),
        "sscratch" => widened(0x140, shadow.scratches[1]),
        "utidc" => Some(shadow.thread_identifiers[0]),
        "stidc" if P::SUPERVISOR => Some(shadow.thread_identifiers[1]),
        "mtidc" => Some(shadow.thread_identifiers[2]),
        _ => {
            let index = name.strip_prefix('x')?.parse::<u8>().ok()?;
            (index < 32).then(|| processor.capability(index))
        }
    }
}

pub(crate) fn describe_capability(
    shadow: &Shadow,
    capability: Capability128,
) -> String {
    let (base, top) = capability.bounds();
    format!(
        "{:#018x} (tag={} base={base:#018x} top={top:#x} perms={:#07x}{})",
        capability.address,
        u8::from(capability.tag),
        shadow.seen_permissions(capability),
        if capability.sealed() { " sealed" } else { "" },
    )
}

fn tags(bus: &Bus, address: &str, count: &str) -> Result<String, &'static str> {
    let number = |text: &str| {
        match text.strip_prefix("0x") {
            Some(hex) => u64::from_str_radix(hex, 16),
            None => text.parse(),
        }
        .map_err(|_| "invalid number")
    };
    let address = number(address)?;
    let count = number(count)?;
    if !address.is_multiple_of(16)
        || !(1..=64).contains(&count)
        || address.checked_add(count * 16).is_none()
    {
        return Err("tags requires sixteen-byte alignment and 1..64 granules \
                    within address space");
    }
    let mut output = String::from("stored tags, at physical addresses:\n");
    for index in 0..count {
        let location = address + index * 16;
        if !bus
            .memory()
            .iter()
            .any(|region| region.contains(location, 16))
        {
            return Err("tag query outside memory");
        }
        let _ = writeln!(
            output,
            "0x{location:016x}: {}",
            u8::from(bus.tag_at(location))
        );
    }
    Ok(output)
}
