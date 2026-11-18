// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod cache;

pub(crate) use cache::DecodeCache;

use super::extension::{Decoding, EXTENSIONS, Extension, ExtensionSet};
use super::instruction::Instruction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedInstruction {
    pub encoded: u32,
    pub instruction: Instruction,
}

impl DecodedInstruction {
    pub const fn length(&self) -> u32 {
        if self.encoded & 3 == 3 { 4 } else { 2 }
    }
}

#[inline(never)]
pub fn compressed(
    extensions: ExtensionSet,
    xlen: u32,
    encoded: u16,
) -> Option<DecodedInstruction> {
    if encoded & 3 == 3 || !extensions.contains(ExtensionSet::C) {
        return None;
    }
    let instruction = first(extensions, |extension| {
        extension.compressed.map(|decode| decode(encoded, xlen))
    })?;
    Some(DecodedInstruction {
        encoded: u32::from(encoded),
        instruction,
    })
}

fn first(
    extensions: ExtensionSet,
    decode: impl Fn(&Extension) -> Option<Decoding>,
) -> Option<Instruction> {
    EXTENSIONS
        .iter()
        .filter(|extension| extensions.contains(extension.set))
        .find_map(|extension| match decode(extension) {
            None | Some(Decoding::Absent) => None,
            Some(Decoding::Found(instruction)) => Some(Some(instruction)),
            Some(Decoding::Reserved) => Some(None),
        })
        .flatten()
}

pub(crate) fn sign_extend(value: u32, bits: u32) -> i32 {
    ((value << (32 - bits)) as i32) >> (32 - bits)
}

#[inline(never)]
pub fn standard(
    extensions: ExtensionSet,
    xlen: u32,
    encoded: u32,
) -> Option<DecodedInstruction> {
    if encoded & 3 != 3 {
        return None;
    }
    let instruction = first(extensions, |extension| {
        extension.standard.map(|decode| decode(encoded, xlen))
    })?;
    Some(DecodedInstruction {
        encoded,
        instruction,
    })
}
