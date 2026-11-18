// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![deny(clippy::indexing_slicing)]

use crate::hart::cheriot_rv32e;
use crate::isa::extension::ExtensionSet;
use crate::machine::bus::Bus;
use crate::machine::elf::{self, Elf};
use crate::machine::error::EmulatorError;
use crate::machine::memory::REGION_LIMIT;
use loadable_segment::LoadableSegment;

mod loadable_segment;

pub(crate) const REQUIRED_KIND: &str =
    "requires little-endian RISC-V ELF32 ET_EXEC";

pub(crate) const REQUIRED_WIDE_KIND: &str =
    "requires little-endian RISC-V ELF64 ET_EXEC";

fn check_flags(
    flags: u32,
    extensions: ExtensionSet,
    allowed_flags: u32,
    required_flags: u32,
) -> Result<(), EmulatorError> {
    if flags & !allowed_flags != 0 || flags & required_flags != required_flags {
        return Err(firmware_error(
            "ELF ABI flags are unsupported or disagree with the selected \
             execution mode",
        ));
    }

    if flags & 1 != 0 && !extensions.contains(ExtensionSet::C) {
        return Err(firmware_error(
            "ELF uses compressed instructions but the processor has C disabled",
        ));
    }

    if (flags & 8 != 0) != extensions.contains(ExtensionSet::E) {
        return Err(firmware_error(
            "ELF RV32E ABI flag disagrees with selected processor",
        ));
    }
    Ok(())
}

pub(crate) fn elf_symbol_address(
    bytes: &[u8],
    name: &str,
) -> Result<u64, EmulatorError> {
    let invalid = |error: String| firmware_error(&error);
    let symbols = Elf::parse(bytes)
        .and_then(|file| file.symbols())
        .map_err(invalid)?
        .ok_or_else(|| {
            firmware_error("completion requires an ELF symbol table")
        })?;
    let mut found = None;
    for symbol in symbols.iter().filter(|symbol| symbol.name == name) {
        if symbol.section == elf::UNDEFINED
            || found.replace(symbol.value).is_some()
        {
            return Err(firmware_error(
                "completion symbol must be uniquely defined",
            ));
        }
    }
    found.ok_or_else(|| {
        firmware_error(&format!("completion requires a {name} symbol"))
    })
}

fn firmware_error(message: &str) -> EmulatorError {
    EmulatorError::Firmware(message.into())
}

pub(crate) fn load_cheriot_elf(
    bus: &mut Bus,
    bytes: &[u8],
) -> Result<u32, EmulatorError> {
    load_elf_with_flags(
        false,
        bus,
        bytes,
        cheriot_rv32e::Processor::EXTENSIONS,
        0x70009,
        0x70009,
    )
    .map(|entry| entry as u32)
}

pub(crate) fn load_elf(
    bus: &mut Bus,
    bytes: &[u8],
    extensions: ExtensionSet,
) -> Result<u32, EmulatorError> {
    load_elf_with_flags(false, bus, bytes, extensions, 0x9, 0)
        .map(|entry| entry as u32)
}

fn load_elf_with_flags(
    wide: bool,
    bus: &mut Bus,
    bytes: &[u8],
    extensions: ExtensionSet,
    allowed_flags: u32,
    required_flags: u32,
) -> Result<u64, EmulatorError> {
    let (class, required_kind, header_size) = if wide {
        (elf::CLASS_64, REQUIRED_WIDE_KIND, 64)
    } else {
        (elf::CLASS_32, REQUIRED_KIND, 52)
    };
    if bytes.get(..4) == Some(&elf::MAGIC[..])
        && (bytes.get(4) != Some(&class)
            || bytes.get(5) != Some(&elf::LITTLE_ENDIAN))
    {
        return Err(firmware_error(required_kind));
    }
    let invalid =
        |error: String| firmware_error(&format!("invalid ELF: {error}"));
    let header = Elf::parse(bytes).map_err(invalid)?;
    header.sections().map_err(invalid)?;
    if header.kind != elf::EXECUTABLE
        || header.machine != elf::RISCV
        || header.version != 1
    {
        return Err(firmware_error(required_kind));
    }
    let entry = header.entry;
    check_flags(header.flags, extensions, allowed_flags, required_flags)?;
    if !header.has_program_headers() {
        return Err(firmware_error("missing program headers"));
    }
    let headers = header.segments().map_err(invalid)?;
    if headers.is_empty()
        || headers.len() > 1024
        || header.header_size != header_size
    {
        return Err(firmware_error(
            "invalid ELF header or program-header count",
        ));
    }
    let mut segments: Vec<LoadableSegment> = Vec::new();

    for segment in headers {
        let kind = segment.kind;
        let alignment = segment.alignment;
        let virtual_address = segment.virtual_address;
        let physical_address = segment.physical_address;
        let offset = segment.offset;
        let permissions = segment.flags;
        if matches!(
            kind,
            elf::SEGMENT_INTERPRETER
                | elf::SEGMENT_DYNAMIC
                | elf::SEGMENT_THREAD_LOCAL
        ) {
            return Err(firmware_error(
                "dynamic linking, interpreters and TLS boot setup are \
                 unsupported",
            ));
        }

        if kind != elf::SEGMENT_LOAD {
            continue;
        }

        if alignment > 1
            && (!alignment.is_power_of_two()
                || virtual_address % alignment != offset % alignment)
        {
            return Err(firmware_error("ELF segment alignment is invalid"));
        }

        if physical_address != 0 && physical_address != virtual_address {
            return Err(firmware_error(
                "different ELF physical and virtual addresses require an \
                 unsupported relocation/boot protocol",
            ));
        }

        if permissions & !7 != 0 {
            return Err(firmware_error("unsupported segment permission bits"));
        }

        let address = virtual_address;
        let file_offset = offset as usize;
        let file_size = segment.file_size as usize;
        let memory_size = segment.memory_size as usize;
        let executable = permissions & elf::SEGMENT_EXECUTABLE != 0;

        if memory_size == 0 && file_size == 0 {
            continue;
        }

        if memory_size < file_size
            || memory_size > REGION_LIMIT
            || memory_size == 0
        {
            return Err(firmware_error(
                "invalid or oversized ELF loadable segment",
            ));
        }

        if address.checked_add(memory_size as u64).is_none() {
            return Err(firmware_error(
                "ELF segment extends outside the address space",
            ));
        }

        slice_at(bytes, file_offset, file_size)?;

        if !bus.memory.iter().any(|memory| {
            memory.contains(address, memory_size)
                && (!executable || memory.executable)
                && (permissions & elf::SEGMENT_WRITABLE == 0
                    || memory.kind == crate::machine::memory::MemoryKind::Ram)
        }) {
            return Err(firmware_error(
                "ELF segment is not fully backed by configured RAM or ROM",
            ));
        }

        let start = address;
        let end = start + memory_size as u64;

        for prior in &segments {
            let other_start = prior.address;
            let other_end = other_start + prior.memory_size as u64;

            if start < other_end && other_start < end {
                return Err(firmware_error(
                    "overlapping ELF loadable segments are unsupported",
                ));
            }
        }

        segments.push(LoadableSegment {
            address,
            file_offset,
            file_size,
            memory_size,
            executable,
        });
    }

    let executable_entry = segments.iter().any(|segment| {
        segment.executable
            && entry >= segment.address
            && entry
                .saturating_add(u64::from(extensions.instruction_alignment()))
                <= segment.address + segment.memory_size as u64
    });

    if !executable_entry {
        return Err(firmware_error(
            "ELF entry point must be in an executable loadable segment",
        ));
    }

    if !entry.is_multiple_of(u64::from(extensions.instruction_alignment())) {
        return Err(firmware_error(
            "ELF entry point is not aligned for this processor",
        ));
    }

    for segment in segments {
        bus.install_firmware_segment(
            segment.address,
            slice_at(bytes, segment.file_offset, segment.file_size)?,
            segment.memory_size,
        )?;
    }

    Ok(entry)
}

pub(crate) fn load_raw(
    bus: &mut Bus,
    address: u64,
    bytes: &[u8],
) -> Result<(), EmulatorError> {
    if bytes.is_empty() {
        return Err(firmware_error("raw firmware must not be empty"));
    }

    bus.install_firmware(address, bytes)
}

pub(crate) fn load_wide_elf(
    bus: &mut Bus,
    bytes: &[u8],
    extensions: ExtensionSet,
) -> Result<u64, EmulatorError> {
    load_elf_with_flags(true, bus, bytes, extensions, 0x9, 0)
}

fn slice_at(
    bytes: &[u8],
    position: usize,
    length: usize,
) -> Result<&[u8], EmulatorError> {
    bytes
        .get(
            position
                ..position
                    .checked_add(length)
                    .ok_or_else(|| firmware_error("ELF offset overflow"))?,
        )
        .ok_or_else(|| firmware_error("truncated ELF header or segment"))
}

pub(crate) fn validate_raw_entry(
    bus: &Bus,
    address: u64,
    bytes: &[u8],
    alignment: u32,
) -> Result<(), EmulatorError> {
    if !address.is_multiple_of(u64::from(alignment))
        || bytes.len() < alignment as usize
    {
        return Err(firmware_error(&format!(
            "raw firmware needs a {alignment}-byte-aligned entry and at least \
             {alignment} bytes"
        )));
    }
    if !bus.memory().iter().any(|memory| {
        memory.executable && memory.contains(address, bytes.len())
    }) {
        return Err(firmware_error(
            "raw firmware must fit in executable memory",
        ));
    }
    Ok(())
}
