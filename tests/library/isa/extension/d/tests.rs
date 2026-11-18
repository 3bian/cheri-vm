// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::{compressed, standard};
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{FloatOperation, FloatWidth, Instruction};

#[test]
fn a_machine_decodes_only_the_widths_and_sizes_it_has() {
    let single = set(&["i", "f", "c", "zicsr"]);
    let both = set(&["i", "f", "d", "c", "zicsr"]);
    let neither = set(&["i", "c", "zicsr"]);
    assert!(standard(neither, 64, 0x68c5_9543).is_none());
    assert!(standard(single, 64, 0x68c5_9543).is_some());
    assert!(standard(single, 64, 0x5a05_f553).is_none());
    assert!(standard(single, 64, 0x4205_8553).is_none());
    assert!(standard(both, 32, 0xc235_b553).is_none());
    assert!(standard(both, 32, 0xd025_f553).is_none());
    assert!(standard(both, 64, 0x68c5_d543).is_none());
    assert!(standard(both, 64, 0x4015_8553).is_some());
    assert!(standard(both, 32, 0xe205_8553).is_none());
    assert!(standard(both, 64, 0xe205_8553).is_some());
}

fn float(
    operation: FloatOperation,
    width: FloatWidth,
    registers: [u8; 4],
    rounding: u8,
) -> Instruction {
    Instruction::Float {
        operation,
        width,
        destination: registers[0],
        first: registers[1],
        second: registers[2],
        third: registers[3],
        rounding,
    }
}

fn load(
    width: FloatWidth,
    destination: u8,
    base: u8,
    offset: i32,
) -> Instruction {
    Instruction::FloatLoad {
        width,
        destination,
        base,
        offset,
    }
}

fn set(names: &[&str]) -> ExtensionSet {
    ExtensionSet::named(names.iter().copied()).unwrap()
}

fn store(width: FloatWidth, source: u8, base: u8, offset: i32) -> Instruction {
    Instruction::FloatStore {
        width,
        source,
        base,
        offset,
    }
}

#[test]
fn the_compressed_loads_and_stores_decode_for_the_width_the_machine_gives_them()
{
    use FloatWidth::*;
    let both = set(&["i", "f", "d", "c", "zicsr"]);
    for (halfword, expected) in [
        (0x2508, load(Double, 10, 10, 8)),
        (0xa80c, store(Double, 11, 8, 16)),
        (0x2662, load(Double, 12, 2, 24)),
        (0xb036, store(Double, 13, 2, 32)),
    ] {
        assert_eq!(
            compressed(both, 64, halfword).unwrap().instruction,
            expected,
            "{halfword:04x}"
        );
    }
    assert!(compressed(set(&["i", "c"]), 64, 0x2508).is_none());
    assert!(compressed(set(&["i", "f", "d", "zicsr"]), 64, 0x2508).is_none());
    assert_eq!(
        compressed(both, 32, 0x6508).unwrap().instruction,
        load(Single, 10, 10, 8)
    );
    assert!(matches!(
        compressed(both, 64, 0x6508).unwrap().instruction,
        Instruction::Load { .. }
    ));
    assert_eq!(
        compressed(both, 32, 0xe02a).unwrap().instruction,
        store(Single, 10, 2, 0)
    );
}

#[test]
fn the_floating_point_encodings_decode_with_their_width_and_rounding() {
    use FloatOperation::*;
    use FloatWidth::*;
    let both = set(&["i", "f", "d", "c", "zicsr"]);
    for (word, expected) in [
        (0x68c5_9543, float(MultiplyAdd, Single, [10, 11, 12, 13], 1)),
        (
            0x6ac5_f54b,
            float(NegatedMultiplySubtract, Double, [10, 11, 12, 13], 7),
        ),
        (0x5a05_f553, float(SquareRoot, Double, [10, 11, 0, 0], 7)),
        (
            0x20c5_a553,
            float(SignInjectXor, Single, [10, 11, 12, 0], 2),
        ),
        (0x2ac5_9553, float(Maximum, Double, [10, 11, 12, 0], 1)),
        (0xa2c5_a553, float(Equal, Double, [10, 11, 12, 0], 2)),
        (0xe005_9553, float(Classify, Single, [10, 11, 0, 0], 1)),
        (
            0xc235_b553,
            float(ToUnsignedLong, Double, [10, 11, 3, 0], 3),
        ),
        (0xd025_f553, float(FromLong, Single, [10, 11, 2, 0], 7)),
        (0x4205_8553, float(Resize, Double, [10, 11, 0, 0], 0)),
        (
            0xf005_8553,
            float(MoveFromInteger, Single, [10, 11, 0, 0], 0),
        ),
        (0x0045_2707, load(Single, 14, 10, 4)),
        (0xfef5_ae27, store(Single, 15, 11, -4)),
    ] {
        assert_eq!(
            standard(both, 64, word).unwrap().instruction,
            expected,
            "{word:08x}"
        );
    }
}
