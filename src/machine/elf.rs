// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![deny(clippy::indexing_slicing)]

mod section;
mod segment;
mod symbol;
mod table;

pub use section::Section;
pub use segment::Segment;
pub use symbol::Symbol;
use table::Table;

pub(crate) const CLASS_32: u8 = 1;
pub(crate) const CLASS_64: u8 = 2;
pub(crate) const EXECUTABLE: u16 = 2;
pub(crate) const LITTLE_ENDIAN: u8 = 1;
pub(crate) const MAGIC: &[u8; 4] = b"\x7fELF";
pub(crate) const RISCV: u16 = 243;
pub(crate) const SECTION_COMPRESSED: u64 = 0x800;
pub(crate) const SECTION_EXECUTABLE: u64 = 0x4;
pub(crate) const SECTION_NO_BITS: u32 = 8;
pub(crate) const SECTION_PROGRAM_BITS: u32 = 1;
pub(crate) const SECTION_SYMBOLS: u32 = 2;
pub(crate) const SEGMENT_DYNAMIC: u32 = 2;
pub(crate) const SEGMENT_EXECUTABLE: u32 = 1;
pub(crate) const SEGMENT_INTERPRETER: u32 = 3;
pub(crate) const SEGMENT_LOAD: u32 = 1;
pub(crate) const SEGMENT_THREAD_LOCAL: u32 = 7;
pub(crate) const SEGMENT_WRITABLE: u32 = 2;
pub(crate) const SYMBOL_FUNCTION: u8 = 2;
pub(crate) const UNDEFINED: u16 = 0;

pub struct Elf<'a> {
    bytes: &'a [u8],
    pub entry: u64,
    pub flags: u32,
    pub header_size: u16,
    pub kind: u16,
    pub machine: u16,
    names: u16,
    program: Table,
    sections: Table,
    pub version: u32,
    pub wide: bool,
}

impl<'a> Elf<'a> {
    pub fn has_program_headers(&self) -> bool {
        self.program.offset != 0
    }

    pub fn parse(bytes: &'a [u8]) -> Result<Self, String> {
        let truncated = || "truncated ELF header".to_owned();
        if bytes.get(..4) != Some(&MAGIC[..]) {
            return Err("not an ELF file".into());
        }
        let wide = match (bytes.get(4), bytes.get(5)) {
            (Some(&CLASS_32), Some(&LITTLE_ENDIAN)) => false,
            (Some(&CLASS_64), Some(&LITTLE_ENDIAN)) => true,
            _ => return Err("unsupported ELF class or byte order".into()),
        };
        let word = |at: usize, size: usize| {
            number(bytes, at, size).ok_or_else(truncated)
        };
        let (address, after) = if wide { (8, 48) } else { (4, 36) };
        let table = |offset: usize, at: usize| -> Result<Table, String> {
            Ok(Table {
                count: word(at + 2, 2)? as usize,
                entry_size: word(at, 2)? as usize,
                offset: word(offset, address)?,
            })
        };
        Ok(Self {
            bytes,
            entry: word(24, address)?,
            flags: word(after, 4)? as u32,
            header_size: word(after + 4, 2)? as u16,
            kind: word(16, 2)? as u16,
            machine: word(18, 2)? as u16,
            names: word(after + 14, 2)? as u16,
            program: table(24 + address, after + 6)?,
            sections: table(24 + 2 * address, after + 10)?,
            version: word(20, 4)? as u32,
            wide,
        })
    }

    fn rows(
        &self,
        table: &Table,
        size: usize,
    ) -> Result<Vec<&'a [u8]>, String> {
        if table.offset == 0 || table.count == 0 {
            return Ok(Vec::new());
        }
        let outside = || "an ELF table is outside the file".to_owned();
        if table.entry_size != size {
            return Err("an ELF table has entries of the wrong size".into());
        }
        let start = usize::try_from(table.offset).map_err(|_| outside())?;
        let length = table.count.checked_mul(size).ok_or_else(outside)?;
        let end = start.checked_add(length).ok_or_else(outside)?;
        let rows = self.bytes.get(start..end).ok_or_else(outside)?;
        Ok(rows.chunks_exact(size).collect())
    }

    pub fn sections(&self) -> Result<Vec<Section<'a>>, String> {
        let rows =
            self.rows(&self.sections, if self.wide { 64 } else { 40 })?;
        let read = |row: &[u8], at: usize, size: usize| {
            number(row, at, size).unwrap_or_default()
        };
        let mut sections: Vec<Section<'a>> = rows
            .iter()
            .map(|row| {
                let (word, link, entry_size) =
                    if self.wide { (8, 40, 56) } else { (4, 24, 36) };
                Section {
                    address: read(row, 8 + word, word),
                    bytes: self.bytes,
                    entry_size: read(row, entry_size, word),
                    flags: read(row, 8, word),
                    kind: read(row, 4, 4) as u32,
                    link: read(row, link, 4) as u32,
                    name: "",
                    name_offset: read(row, 0, 4) as usize,
                    offset: read(row, 8 + 2 * word, word),
                    size: read(row, 8 + 3 * word, word),
                }
            })
            .collect();
        let names = sections
            .get(usize::from(self.names))
            .and_then(Section::data)
            .unwrap_or_default();
        for section in &mut sections {
            section.name = text(names, section.name_offset);
        }
        Ok(sections)
    }

    pub fn segments(&self) -> Result<Vec<Segment>, String> {
        let rows = self.rows(&self.program, if self.wide { 56 } else { 32 })?;
        let read = |row: &[u8], at: usize, size: usize| {
            number(row, at, size).unwrap_or_default()
        };
        Ok(rows
            .iter()
            .map(|row| {
                if self.wide {
                    Segment {
                        alignment: read(row, 48, 8),
                        file_size: read(row, 32, 8),
                        flags: read(row, 4, 4) as u32,
                        kind: read(row, 0, 4) as u32,
                        memory_size: read(row, 40, 8),
                        offset: read(row, 8, 8),
                        physical_address: read(row, 24, 8),
                        virtual_address: read(row, 16, 8),
                    }
                } else {
                    Segment {
                        alignment: read(row, 28, 4),
                        file_size: read(row, 16, 4),
                        flags: read(row, 24, 4) as u32,
                        kind: read(row, 0, 4) as u32,
                        memory_size: read(row, 20, 4),
                        offset: read(row, 4, 4),
                        physical_address: read(row, 12, 4),
                        virtual_address: read(row, 8, 4),
                    }
                }
            })
            .collect())
    }

    pub fn symbols(&self) -> Result<Option<Vec<Symbol<'a>>>, String> {
        let sections = self.sections()?;
        let Some(table) = sections
            .iter()
            .find(|section| section.kind == SECTION_SYMBOLS)
        else {
            return Ok(None);
        };
        let size = if self.wide { 24 } else { 16 };
        let outside = || "the ELF symbol table is outside the file".to_owned();
        let data = table.data().ok_or_else(outside)?;
        let names = sections
            .get(table.link as usize)
            .and_then(Section::data)
            .ok_or_else(outside)?;
        let read = |row: &[u8], at: usize, size: usize| {
            number(row, at, size).unwrap_or_default()
        };
        Ok(Some(
            data.chunks_exact(size)
                .map(|row| {
                    let name = text(names, read(row, 0, 4) as usize);
                    if self.wide {
                        Symbol {
                            kind: read(row, 4, 1) as u8 & 0xf,
                            name,
                            section: read(row, 6, 2) as u16,
                            size: read(row, 16, 8),
                            value: read(row, 8, 8),
                        }
                    } else {
                        Symbol {
                            kind: read(row, 12, 1) as u8 & 0xf,
                            name,
                            section: read(row, 14, 2) as u16,
                            size: read(row, 8, 4),
                            value: read(row, 4, 4),
                        }
                    }
                })
                .collect(),
        ))
    }
}

fn number(bytes: &[u8], at: usize, size: usize) -> Option<u64> {
    let field = bytes.get(at..at.checked_add(size)?)?;
    let mut value = [0_u8; 8];
    value.get_mut(..size)?.copy_from_slice(field);
    Some(u64::from_le_bytes(value))
}

fn text(table: &[u8], offset: usize) -> &str {
    let rest = table.get(offset..).unwrap_or_default();
    let end = rest
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(rest.len());
    rest.get(..end)
        .and_then(|name| core::str::from_utf8(name).ok())
        .unwrap_or_default()
}
