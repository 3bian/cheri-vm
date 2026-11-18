// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::{compressed, standard};
use crate::isa::extension::ExtensionSet;
use crate::isa::extension::rvy::Operation;
use crate::isa::instruction::{
    AtomicOperation, AtomicWidth, Instruction, LoadKind, StoreKind,
};

#[test]
fn a_short_form_that_would_leave_a_register_as_it_was_is_reserved() {
    for halfword in [0x0081, 0x0082, 0x8001, 0x8401] {
        assert_eq!(short(halfword), None, "{halfword:#x}");
        assert_eq!(
            compressed(set(&["i", "c"]), 64, halfword)
                .map(|decoded| decoded.instruction),
            Some(Instruction::Hint)
        );
    }
    assert_eq!(short(0x0001), Some(Instruction::Hint));
}

#[test]
fn capability_loads_stores_and_atomics_decode_and_refuse_the_zero_register_as_a_base()
 {
    assert_eq!(
        long(0xff0 << 20 | 2 << 15 | 1 << 12 | 1 << 7 | 0x7b),
        Some(Instruction::Load {
            kind: LoadKind::Capability,
            destination: 1,
            base: 2,
            offset: -16,
        })
    );
    assert_eq!(long(1 << 12 | 1 << 7 | 0x7b), None);
    assert_eq!(
        long(1 << 25 | 3 << 20 | 2 << 15 | 2 << 12 | 0x10 << 7 | 0x7b),
        Some(Instruction::Store {
            kind: StoreKind::Capability,
            source: 3,
            base: 2,
            offset: 48,
        })
    );
    assert_eq!(long(3 << 20 | 2 << 12 | 0x7b), None);
    let atomic = |operation, source| {
        Some(Instruction::Atomic {
            operation,
            width: AtomicWidth::Capability,
            destination: 1,
            address: 2,
            source,
        })
    };
    let encoded = |code: u32, source: u32| {
        code << 27 | source << 20 | 2 << 15 | 3 << 12 | 1 << 7 | 0x7b
    };
    assert_eq!(
        long(encoded(2, 0)),
        atomic(AtomicOperation::LoadReserved, 0)
    );
    assert_eq!(long(encoded(2, 3)), None);
    assert_eq!(
        long(encoded(3, 3)),
        atomic(AtomicOperation::StoreConditional, 3)
    );
    assert_eq!(long(encoded(1, 3)), atomic(AtomicOperation::Swap, 3));
    assert_eq!(long(encoded(0, 3)), None);
}

fn long(word: u32) -> Option<Instruction> {
    standard(set(&["y", "i", "c"]), 64, word).map(|decoded| decoded.instruction)
}

fn register(prefix: u32, second: u32, first: u32) -> u32 {
    prefix << 25 | second << 20 | first << 15 | 1 << 7 | 0x7b
}

fn rvy(
    operation: Operation,
    first: u8,
    second: u8,
    immediate: i16,
) -> Option<Instruction> {
    Some(Instruction::Rvy {
        operation,
        destination: 1,
        first,
        second,
        immediate,
    })
}

fn set(names: &[&str]) -> ExtensionSet {
    ExtensionSet::named(names.iter().copied()).unwrap()
}

fn short(halfword: u16) -> Option<Instruction> {
    compressed(set(&["y", "i", "c", "f", "d", "zicsr"]), 64, halfword)
        .map(|decoded| decoded.instruction)
}

#[test]
fn the_immediate_forms_decode_and_a_bounds_length_is_spread_over_nine_bits() {
    use Operation::*;
    let immediate = |value: u32, function_code: u32| {
        value << 20 | 2 << 15 | function_code << 12 | 1 << 7 | 0x7b
    };
    assert_eq!(long(immediate(0xff0, 4)), rvy(AddImmediate, 2, 0, -16));
    assert_eq!(long(immediate(0x7ff, 4)), rvy(AddImmediate, 2, 0, 2047));
    for (encoded, length) in [
        (0, 4096),
        (1, 1),
        (100, 100),
        (255, 255),
        (0x100, 256),
        (0x101, 272),
        (0x110, 264),
        (0x11f, 504),
        (0x120, 512),
        (0x121, 528),
        (0x1ff, 4080),
    ] {
        assert_eq!(
            long(immediate(0xe00 | encoded, 5)),
            rvy(SetBoundsImmediate, 2, 0, length),
            "{encoded:#x}"
        );
    }
    assert_eq!(long(immediate(0xc00, 5)), None);
    assert_eq!(long(immediate(0, 6)), None);
}

#[test]
fn the_register_forms_decode_from_the_table_of_the_draft() {
    use Operation::*;
    for (prefix, operation) in [
        (0x01, SetHigh),
        (0x03, Add),
        (0x06, Equal),
        (0x07, Unseal),
        (0x0b, SetAddress),
        (0x0e, Subset),
        (0x0f, Build),
        (0x13, ClearPermissions),
        (0x1b, SetBounds),
        (0x23, SetBoundsRounded),
    ] {
        assert_eq!(long(register(prefix, 3, 2)), rvy(operation, 2, 3, 0));
    }
    assert_eq!(long(register(0x03, 0, 2)), rvy(Move, 2, 0, 0));
    assert_eq!(long(register(0x17, 3, 0)), rvy(SealEntry, 3, 0, 0));
    assert_eq!(long(register(0x17, 3, 2)), None);
    for (code, operation) in [
        (0, Base),
        (1, Permissions),
        (2, Top),
        (3, Length),
        (4, Tag),
        (5, Type),
        (7, High),
    ] {
        assert_eq!(long(register(0x7a, code, 2)), rvy(operation, 2, 0, 0));
    }
    assert_eq!(long(register(0x7a, 6, 2)), None);
    assert_eq!(long(register(0x78, 0, 2)), rvy(AlignmentMask, 2, 0, 0));
    assert_eq!(long(register(0x78, 1, 2)), None);
    assert_eq!(long(register(0x05, 3, 2)), rvy(ShiftAdd, 2, 3, 1));
    assert_eq!(long(register(0x1d, 3, 2)), rvy(ShiftAdd, 2, 3, 4));
    assert_eq!(
        long(register(0x3d, 3, 2)),
        rvy(ShiftAddUnsignedWord, 2, 3, 4)
    );
    assert_eq!(long(register(0x02, 3, 2)), None);
    assert_eq!(standard(set(&["i", "c"]), 64, register(0x03, 3, 2)), None);
}

#[test]
fn the_short_forms_move_capabilities_and_take_the_place_of_the_double_precision_ones()
 {
    use Operation::*;
    let add = |destination, immediate| {
        Some(Instruction::Rvy {
            operation: AddImmediate,
            destination,
            first: 2,
            second: 0,
            immediate,
        })
    };
    assert_eq!(short(0x808e), rvy(Move, 3, 0, 0));
    assert_eq!(short(0x6105), add(2, 32));
    assert_eq!(short(0x7101), add(2, -512));
    assert_eq!(short(0x0800), add(8, 16));
    let load = |destination, base, offset| {
        Some(Instruction::Load {
            kind: LoadKind::Capability,
            destination,
            base,
            offset,
        })
    };
    let store = |source, base, offset| {
        Some(Instruction::Store {
            kind: StoreKind::Capability,
            source,
            base,
            offset,
        })
    };
    assert_eq!(short(0x2904), load(9, 10, 16));
    assert_eq!(short(0x3082), load(1, 2, 32));
    assert_eq!(short(0x2002), None);
    assert_eq!(short(0xa08e), store(3, 2, 64));
    assert_eq!(short(0xa504), store(9, 10, 256));
}
