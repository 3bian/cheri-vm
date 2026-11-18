// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::fmt::Write;

use crate::hart::cheriot_rv32e::Processor;
use crate::isa::pointer::cheriot::Capability;
use crate::machine::bus::Bus;

fn capability(processor: &Processor, name: &str) -> Option<Capability> {
    Some(match name {
        "pcc" => processor.pcc,
        "mtcc" => processor.machine_trap,
        "mtdc" => processor.machine_data,
        "mscratchc" => processor.machine_scratch,
        "mepcc" => processor.machine_exception,
        _ => *processor
            .registers()
            .get(name.strip_prefix('c')?.parse::<usize>().ok()?)?,
    })
}

pub(super) fn describe_capability(
    processor: &Processor,
    name: &str,
) -> Result<String, &'static str> {
    let capability =
        capability(processor, name).ok_or("unknown capability register")?;
    let (payload, tag) = capability.to_memory();
    let (base, top) = capability.bounds();
    Ok(format!(
        "{name}: payload=0x{payload:016x} tag={} cursor=0x{:08x} \
         base=0x{base:08x} top=0x{top:09x} permissions=0x{:03x} sealed={} \
         type={}\n",
        u8::from(tag),
        capability.address(),
        capability.permissions(),
        capability.sealed(),
        capability.object_type()
    ))
}

pub(super) fn tags(
    bus: &Bus,
    address: &str,
    count: &str,
) -> Result<String, &'static str> {
    let number = |text: &str| {
        match text.strip_prefix("0x") {
            Some(hex) => u32::from_str_radix(hex, 16),
            None => text.parse(),
        }
        .map_err(|_| "invalid number")
    };
    let address = number(address)?;
    let count = number(count)?;
    if !address.is_multiple_of(8)
        || !(1..=64).contains(&count)
        || u64::from(address) + u64::from(count) * 8 > 1_u64 << 32
    {
        return Err("tags requires eight-byte alignment and 1..64 granules \
                    within address space");
    }
    let mut output =
        String::from("stored tags (without revocation filtering):\n");
    for index in 0..count {
        let location = address + index * 8;
        let memory = bus
            .memory()
            .iter()
            .find(|region| region.contains(location.into(), 8))
            .ok_or("tag query outside memory")?;
        let tag = memory
            .tag_at(location.into())
            .ok_or("memory is not tagged")?;
        let _ = writeln!(output, "0x{location:08x}: {}", u8::from(tag));
    }
    Ok(output)
}
