// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use super::{zbb, zbc};
use crate::isa::instruction::{
    AesByteOperation, ImmediateOperation, Instruction, RegisterOperation,
};

fn bits(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    let fields = Fields::of(word, xlen);
    Some(match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 0x04, 4) => fields.register(Pack),
        (0x33, 0x04, 7) => fields.register(PackHigh),
        (0x3b, 0x04, 4) if fields.wide => fields.register_word(Pack),
        (0x33, 0x14, 2) => fields.register(Permute4),
        (0x33, 0x14, 4) => fields.register(Permute8),
        (0x13, _, 5) if fields.selector == 0x687 => {
            fields.immediate(ImmediateOperation::BitReverseBytes, 0)
        }
        (0x13, _, 1) if !fields.wide && fields.selector == 0x08f => {
            fields.immediate(ImmediateOperation::Zip, 0)
        }
        (0x13, _, 5) if !fields.wide && fields.selector == 0x08f => {
            fields.immediate(ImmediateOperation::Unzip, 0)
        }
        _ => return None,
    })
}

fn hashes(word: u32, xlen: u32) -> Option<Instruction> {
    use ImmediateOperation::*;
    let fields = Fields::of(word, xlen);
    if fields.opcode != 0x13 || fields.function_code != 1 {
        return None;
    }
    Some(fields.immediate(
        match fields.selector {
            0x100 => Sha256Sum0,
            0x101 => Sha256Sum1,
            0x102 => Sha256Sigma0,
            0x103 => Sha256Sigma1,
            0x104 if fields.wide => Sha512Sum0,
            0x105 if fields.wide => Sha512Sum1,
            0x106 if fields.wide => Sha512Sigma0,
            0x107 if fields.wide => Sha512Sigma1,
            _ => return None,
        },
        0,
    ))
}

fn narrow_cipher(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    let fields = Fields::of(word, xlen);
    if fields.opcode != 0x33 || fields.function_code != 0 {
        return None;
    }
    let byte = |operation| Instruction::AesByte {
        operation,
        destination: fields.destination,
        first: fields.first,
        second: fields.second,
        select: (word >> 30) as u8,
    };
    Some(match fields.prefix {
        0x2e => fields.register(Sha512Sigma0High),
        0x2a => fields.register(Sha512Sigma0Low),
        0x2f => fields.register(Sha512Sigma1High),
        0x2b => fields.register(Sha512Sigma1Low),
        0x28 => fields.register(Sha512Sum0Rotated),
        0x29 => fields.register(Sha512Sum1Rotated),
        prefix => match prefix & 0x1f {
            0x15 => byte(AesByteOperation::Decrypt),
            0x17 => byte(AesByteOperation::DecryptMix),
            0x11 => byte(AesByteOperation::Encrypt),
            0x13 => byte(AesByteOperation::EncryptMix),
            _ => return None,
        },
    })
}

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    zbb::rotations(word, xlen)
        .or_else(|| zbc::multiplies(word, xlen))
        .or_else(|| bits(word, xlen))
        .or_else(|| hashes(word, xlen))
        .or_else(|| {
            if xlen == 64 {
                wide_cipher(word, xlen)
            } else {
                narrow_cipher(word, xlen)
            }
        })
}

fn wide_cipher(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    let fields = Fields::of(word, xlen);
    Some(match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 0x1d, 0) => fields.register(AesDecrypt),
        (0x33, 0x1f, 0) => fields.register(AesDecryptMix),
        (0x33, 0x19, 0) => fields.register(AesEncrypt),
        (0x33, 0x1b, 0) => fields.register(AesEncryptMix),
        (0x33, 0x3f, 0) => fields.register(AesKeySchedule2),
        (0x13, _, 1) if fields.selector == 0x300 => {
            fields.immediate(ImmediateOperation::AesInverseMix, 0)
        }
        (0x13, _, 1)
            if fields.selector >> 4 == 0x31 && fields.selector & 15 <= 10 =>
        {
            fields.immediate(
                ImmediateOperation::AesKeySchedule1,
                (fields.selector & 15) as i32,
            )
        }
        _ => return None,
    })
}
