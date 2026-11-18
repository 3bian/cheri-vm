// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::standard;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{
    AesByteOperation, ImmediateOperation, Instruction, RegisterOperation,
};

fn byte(operation: AesByteOperation, select: u8) -> Instruction {
    Instruction::AesByte {
        operation,
        destination: 1,
        first: 2,
        second: 3,
        select,
    }
}

fn check(name: &str, xlen: u32, rows: &[(u32, Instruction)]) {
    for (word, instruction) in rows {
        assert_eq!(
            decoded(&["i", name], xlen, *word),
            Some(*instruction),
            "{word:#x}"
        );
        assert_eq!(decoded(&["i"], xlen, *word), None, "{word:#x}");
    }
}

fn decoded(names: &[&str], xlen: u32, word: u32) -> Option<Instruction> {
    standard(set(names), xlen, word).map(|decoded| decoded.instruction)
}

fn immediate(operation: ImmediateOperation, immediate: i32) -> Instruction {
    Instruction::Immediate {
        operation,
        destination: 1,
        source: 2,
        immediate,
    }
}

fn register(operation: RegisterOperation, second: u8) -> Instruction {
    Instruction::Register {
        operation,
        destination: 1,
        first: 2,
        second,
    }
}

fn register_word(operation: RegisterOperation, second: u8) -> Instruction {
    Instruction::RegisterWord {
        operation,
        destination: 1,
        first: 2,
        second,
    }
}

fn set(names: &[&str]) -> ExtensionSet {
    ExtensionSet::named(names.iter().copied()).unwrap()
}

#[test]
fn the_32_bit_machine_has_its_own_ciphers_and_the_halves_of_sha_512() {
    use ImmediateOperation::*;
    use RegisterOperation::*;
    check(
        "zkn",
        32,
        &[
            (0x08f1_1093, immediate(Zip, 0)),
            (0x08f1_5093, immediate(Unzip, 0)),
            (0xaa31_00b3, byte(AesByteOperation::Decrypt, 2)),
            (0xee31_00b3, byte(AesByteOperation::DecryptMix, 3)),
            (0x6231_00b3, byte(AesByteOperation::Encrypt, 1)),
            (0x2631_00b3, byte(AesByteOperation::EncryptMix, 0)),
            (0x5c31_00b3, register(Sha512Sigma0High, 3)),
            (0x5431_00b3, register(Sha512Sigma0Low, 3)),
            (0x5e31_00b3, register(Sha512Sigma1High, 3)),
            (0x5631_00b3, register(Sha512Sigma1Low, 3)),
            (0x5031_00b3, register(Sha512Sum0Rotated, 3)),
            (0x5231_00b3, register(Sha512Sum1Rotated, 3)),
            (0x1021_1093, immediate(Sha256Sigma0, 0)),
        ],
    );
    for word in [0x3a31_00b3, 0x3001_1093, 0x1061_1093, 0x0831_40bb] {
        assert_eq!(decoded(&["i", "zkn"], 32, word), None, "{word:#x}");
    }
}

#[test]
fn the_cryptography_instructions_decode_at_64_bits() {
    use ImmediateOperation::*;
    use RegisterOperation::*;
    check(
        "zkn",
        64,
        &[
            (0x4031_70b3, register(AndNot, 3)),
            (0x6031_10b3, register(RotateLeft, 3)),
            (0x62d1_5093, immediate(ImmediateOperation::RotateRight, 45)),
            (0x6b81_5093, immediate(ByteReverse, 0)),
            (0x0a31_10b3, register(CarrylessMul, 3)),
            (0x0a31_30b3, register(CarrylessMulHigh, 3)),
            (0x0831_40b3, register(Pack, 3)),
            (0x0831_70b3, register(PackHigh, 3)),
            (0x0831_40bb, register_word(Pack, 3)),
            (0x6871_5093, immediate(BitReverseBytes, 0)),
            (0x2831_20b3, register(Permute4, 3)),
            (0x2831_40b3, register(Permute8, 3)),
            (0x3a31_00b3, register(AesDecrypt, 3)),
            (0x3e31_00b3, register(AesDecryptMix, 3)),
            (0x3231_00b3, register(AesEncrypt, 3)),
            (0x3631_00b3, register(AesEncryptMix, 3)),
            (0x3001_1093, immediate(AesInverseMix, 0)),
            (0x3171_1093, immediate(AesKeySchedule1, 7)),
            (0x7e31_00b3, register(AesKeySchedule2, 3)),
            (0x1021_1093, immediate(Sha256Sigma0, 0)),
            (0x1031_1093, immediate(Sha256Sigma1, 0)),
            (0x1001_1093, immediate(Sha256Sum0, 0)),
            (0x1011_1093, immediate(Sha256Sum1, 0)),
            (0x1061_1093, immediate(Sha512Sigma0, 0)),
            (0x1071_1093, immediate(Sha512Sigma1, 0)),
            (0x1041_1093, immediate(Sha512Sum0, 0)),
            (0x1051_1093, immediate(Sha512Sum1, 0)),
        ],
    );
    assert_eq!(decoded(&["i", "zkn"], 64, 0x31b1_1093), None);
    assert_eq!(decoded(&["i", "zkn"], 64, 0x0a31_20b3), None);
    assert_eq!(decoded(&["i", "zkn"], 64, 0x08f1_1093), None);
    assert_eq!(decoded(&["i", "zkn"], 64, 0xaa31_00b3), None);
}
