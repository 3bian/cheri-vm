// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::a::*;
use crate::isa::instruction::{AtomicOperation, AtomicWidth, Instruction};

#[test]
fn a_doubleword_needs_64_bits_and_a_load_reserved_names_no_source() {
    assert!(standard(0x1000_b1af, 32).is_none());
    assert!(standard(0x1020_b1af, 64).is_none());
    assert!(standard(0x1000_91af, 64).is_none());
    assert!(standard(0x2800_a1af, 64).is_none());
    assert!(standard(0x0020_81b3, 64).is_none());
    assert_eq!(
        (AtomicWidth::Word.bytes(), AtomicWidth::Doubleword.bytes()),
        (4, 8)
    );
}

#[test]
fn the_atomic_encodings_decode_with_their_width() {
    assert_eq!(
        standard(0x1000_b1af, 64),
        Some(Instruction::Atomic {
            operation: AtomicOperation::LoadReserved,
            width: AtomicWidth::Doubleword,
            destination: 3,
            address: 1,
            source: 0,
        })
    );
    assert_eq!(
        standard(0x1829_28af, 32),
        Some(Instruction::Atomic {
            operation: AtomicOperation::StoreConditional,
            width: AtomicWidth::Word,
            destination: 17,
            address: 18,
            source: 2,
        })
    );
    for (word, operation) in [
        (0x0080_a4af, AtomicOperation::Add),
        (0x0840_b7af, AtomicOperation::Swap),
        (0x8040_a6af, AtomicOperation::Minimum),
        (0xe080_b5af, AtomicOperation::MaximumUnsigned),
    ] {
        assert!(
            matches!(
                standard(word, 64),
                Some(Instruction::Atomic { operation: found, .. })
                    if found == operation
            ),
            "{word:#010x}"
        );
    }
}
