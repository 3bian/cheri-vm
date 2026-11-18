// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::*;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{Instruction, RegisterOperation};

#[test]
fn a_16_bit_encoding_means_what_its_word_size_says() {
    let set = ExtensionSet::I.with(ExtensionSet::C);
    assert!(matches!(
        compressed(set, 32, 0x347d).unwrap().instruction,
        Instruction::JumpAndLink { destination: 1, .. }
    ));
    assert!(matches!(
        compressed(set, 64, 0x347d).unwrap().instruction,
        Instruction::ImmediateWord { destination: 8, .. }
    ));
    assert!(compressed(set, 32, 0x6088).is_none());
    assert!(compressed(set, 64, 0x6088).is_some());
    assert!(compressed(set, 32, 0x1422).is_none());
    assert_eq!(
        compressed(set, 64, 0x1422).unwrap().instruction,
        Instruction::Immediate {
            operation: crate::isa::instruction::ImmediateOperation::ShiftLeft,
            destination: 8,
            source: 8,
            immediate: 40,
        }
    );
}

#[test]
fn an_instruction_decodes_only_when_its_extension_is_in_the_set() {
    let base = ExtensionSet::I;
    let cases = [
        (0x3000_2073, ExtensionSet::ZICSR),
        (0x3020_0073, ExtensionSet::ZICSR),
        (0x1050_0073, ExtensionSet::ZICSR),
        (0x0000_100f, ExtensionSet::ZIFENCEI),
        (0x0220_81b3, ExtensionSet::M),
    ];
    for (encoded, extension) in cases {
        assert!(standard(base, 32, encoded).is_none(), "{encoded:#010x}");
        assert!(
            standard(base.with(extension), 32, encoded).is_some(),
            "{encoded:#010x}"
        );
    }
    assert!(compressed(base, 32, 0x0001).is_none());
    assert!(compressed(base.with(ExtensionSet::C), 32, 0x0001).is_some());
    assert_eq!(
        standard(base, 32, 0x0000_0073).unwrap().instruction,
        Instruction::EnvironmentCall
    );
    assert_eq!(
        standard(base, 32, 0x0000_000f).unwrap().instruction,
        Instruction::Fence
    );
}

#[test]
fn an_instruction_is_eight_bytes_so_it_travels_in_a_register() {
    assert_eq!(size_of::<Instruction>(), 8);
    assert_eq!(size_of::<DecodedInstruction>(), 12);
    assert_eq!(
        standard(ExtensionSet::I, 32, 0x0000_0013).unwrap().length(),
        4
    );
    assert_eq!(compressed(ExtensionSet::C, 32, 0x0001).unwrap().length(), 2);
}

#[test]
fn known_encodings_decode_and_m_needs_its_extension() {
    let base = ExtensionSet::I;
    let with_multiply = base.with(ExtensionSet::M);
    let add = standard(base, 32, 0x002081b3).unwrap();
    assert_eq!(
        add.instruction,
        Instruction::Register {
            operation: RegisterOperation::Add,
            destination: 3,
            first: 1,
            second: 2,
        }
    );
    assert!(standard(base, 32, 0x022081b3).is_none());
    assert!(standard(with_multiply, 32, 0x022081b3).is_some());
    assert!(standard(with_multiply, 32, 0xffff_ffff).is_none());
}

#[test]
fn sign_extension_reaches_both_extremes() {
    assert_eq!(sign_extend(0xfff, 12), -1);
    assert_eq!(sign_extend(0x800, 12), -2048);
    assert_eq!(sign_extend(0x7ff, 12), 2047);
}

#[test]
fn the_64_bit_instructions_decode_only_at_64_bits() {
    use crate::isa::instruction::{ImmediateOperation, LoadKind, StoreKind};
    let set = ExtensionSet::I
        .with(ExtensionSet::M)
        .with(ExtensionSet::A)
        .with(ExtensionSet::C);
    for word in [
        0x0000_b183,
        0x0040_e203,
        0x0020_b023,
        0x0010_811b,
        0x0010_81bb,
        0x0210_833b,
        0x1000_b1af,
        0x0210_d093,
    ] {
        assert!(standard(set, 32, word).is_none(), "{word:#010x}");
        assert!(standard(set, 64, word).is_some(), "{word:#010x}");
    }
    assert!(matches!(
        standard(set, 64, 0x0000_b183).unwrap().instruction,
        Instruction::Load {
            kind: LoadKind::Doubleword,
            ..
        }
    ));
    assert!(matches!(
        standard(set, 64, 0x0020_b023).unwrap().instruction,
        Instruction::Store {
            kind: StoreKind::Doubleword,
            ..
        }
    ));
    assert_eq!(
        standard(set, 64, 0x0210_d093).unwrap().instruction,
        Instruction::Immediate {
            operation: ImmediateOperation::ShiftRightLogical,
            destination: 1,
            source: 1,
            immediate: 33,
        }
    );
    assert_eq!(
        standard(set, 64, 0x41f1_529b).unwrap().instruction,
        Instruction::ImmediateWord {
            operation: ImmediateOperation::ShiftRightArithmetic,
            destination: 5,
            source: 2,
            immediate: 31,
        }
    );
    assert_eq!(
        standard(set, 64, 0x0210_833b).unwrap().instruction,
        Instruction::RegisterWord {
            operation: RegisterOperation::Mul,
            destination: 6,
            first: 1,
            second: 1,
        }
    );
    assert!(standard(set, 64, 0x0210_933b).is_none());
    assert!(standard(set, 64, 0x0210_941b).is_none());
    assert!(standard(ExtensionSet::I, 64, 0x0210_833b).is_none());
    assert!(standard(ExtensionSet::I, 64, 0x1000_b1af).is_none());
}

#[test]
fn the_cheriot_extension_takes_the_jumps_and_leaves_the_rest_to_the_base() {
    let cheriot = ExtensionSet::CHERIOT
        .with(ExtensionSet::E)
        .with(ExtensionSet::M)
        .with(ExtensionSet::C);
    assert!(matches!(
        standard(cheriot, 32, 0x0000_00ef).unwrap().instruction,
        Instruction::Capability { .. }
    ));
    assert!(matches!(
        standard(cheriot, 32, 0x0020_81b3).unwrap().instruction,
        Instruction::Register { .. }
    ));
    assert!(matches!(
        compressed(cheriot, 32, 0x8082).unwrap().instruction,
        Instruction::Capability { .. }
    ));
    assert!(matches!(
        compressed(cheriot, 32, 0x0085).unwrap().instruction,
        Instruction::Immediate { .. }
    ));
}

#[test]
fn what_cheriot_reserves_does_not_fall_through_to_the_base() {
    let cheriot = ExtensionSet::CHERIOT
        .with(ExtensionSet::E)
        .with(ExtensionSet::M)
        .with(ExtensionSet::C);
    for word in [0x0000_08ef, 0x0008_0067, 0x0000_0897] {
        assert!(standard(cheriot, 32, word).is_none(), "{word:#010x}");
        assert!(
            standard(ExtensionSet::I, 32, word).is_some(),
            "{word:#010x}"
        );
    }
    assert!(compressed(cheriot, 32, 0x8802).is_none());
    assert!(compressed(ExtensionSet::C, 32, 0x8802).is_some());
}
