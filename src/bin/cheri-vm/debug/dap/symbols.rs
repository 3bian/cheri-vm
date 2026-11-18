// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod row;

use row::Row;

use cheri_vm::machine::elf::Elf;

use std::collections::BTreeMap;

use std::path::{Path, PathBuf};

#[derive(Default)]
pub(crate) struct Symbols {
    compressed: bool,
    files: Vec<PathBuf>,
    functions: Vec<(u64, u64, String)>,
    rows: Vec<Row>,
}

impl Symbols {
    pub(super) fn addresses(&self, source: &Path, line: u64) -> Vec<u64> {
        let matches = |exact: bool| -> Vec<u64> {
            let mut found: Vec<u64> = self
                .rows
                .iter()
                .filter(|row| row.line == line)
                .filter(|row| {
                    let file = &self.files[row.file];
                    if exact {
                        file == source
                    } else {
                        file.file_name() == source.file_name()
                    }
                })
                .map(|row| row.address)
                .collect();
            found.sort_unstable();
            found.dedup();
            found
        };
        let exact = matches(true);
        if exact.is_empty() {
            matches(false)
        } else {
            exact
        }
    }

    pub(crate) fn compressed(&self) -> bool {
        self.compressed
    }

    pub(super) fn function(&self, address: u64) -> Option<&(u64, u64, String)> {
        let index = self
            .functions
            .partition_point(|(start, _, _)| *start <= address);
        self.functions
            .get(index.checked_sub(1)?)
            .filter(|(_, end, _)| address < *end)
    }

    pub(super) fn line(&self, address: u64) -> Option<(&Path, u64)> {
        let index = self.rows.partition_point(|row| row.address <= address);
        let row = self.rows.get(index.checked_sub(1)?)?;
        (address - row.address < 256)
            .then(|| (self.files[row.file].as_path(), row.line))
    }

    pub(crate) fn load(data: &[u8]) -> Self {
        let mut symbols = Self::default();
        let Ok(file) = Elf::parse(data) else {
            return symbols;
        };
        symbols.functions = file
            .symbols()
            .ok()
            .flatten()
            .unwrap_or_default()
            .iter()
            .filter(|symbol| symbol.is_function())
            .filter_map(|symbol| {
                let start = symbol.value;
                let end = start.checked_add(symbol.size)?;
                Some((start, end, symbol.name.to_owned()))
            })
            .collect();
        symbols.functions.sort();
        let sections = file.sections().unwrap_or_default();
        symbols.compressed = sections.iter().any(|section| {
            (section.name.starts_with(".debug_")
                || section.name.starts_with(".zdebug_"))
                && section.is_compressed()
        });
        let _ = symbols.load_lines(&sections);
        symbols.rows.sort_by_key(|row| row.address);
        symbols
    }

    fn load_lines(
        &mut self,
        sections: &[cheri_vm::machine::elf::Section],
    ) -> std::result::Result<(), gimli::Error> {
        let endian = gimli::RunTimeEndian::Little;
        let load = |id: gimli::SectionId| -> std::result::Result<
            std::borrow::Cow<[u8]>,
            gimli::Error,
        > {
            Ok(sections
                .iter()
                .find(|section| section.name == id.name())
                .filter(|section| !section.is_compressed())
                .and_then(|section| section.data())
                .unwrap_or_default()
                .into())
        };
        let sections = gimli::DwarfSections::load(load)?;
        let dwarf =
            sections.borrow(|section| gimli::EndianSlice::new(section, endian));
        let mut indices: BTreeMap<PathBuf, usize> = BTreeMap::new();
        let mut units = dwarf.units();
        while let Some(header) = units.next()? {
            let unit = dwarf.unit(header)?;
            let Some(program) = unit.line_program.clone() else {
                continue;
            };
            let directory = unit
                .comp_dir
                .map(|dir| PathBuf::from(dir.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row()? {
                if row.end_sequence() || !row.is_stmt() {
                    continue;
                }
                let (Some(line), Some(entry)) = (row.line(), row.file(header))
                else {
                    continue;
                };
                let mut path = directory.clone();
                if let Some(dir) = entry.directory(header)
                    && let Ok(dir) = dwarf.attr_string(&unit, dir)
                {
                    path.push(dir.to_string_lossy().as_ref());
                }
                if let Ok(name) = dwarf.attr_string(&unit, entry.path_name()) {
                    path.push(name.to_string_lossy().as_ref());
                }
                let address = row.address();
                if address == 0 {
                    continue;
                }
                let next = self.files.len();
                let index = *indices.entry(path.clone()).or_insert(next);
                if index == next {
                    self.files.push(path);
                }
                self.rows.push(Row {
                    address,
                    file: index,
                    line: line.get(),
                });
            }
        }
        Ok(())
    }
}
