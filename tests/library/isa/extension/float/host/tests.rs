// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::float::arithmetic::{
    add, divide, multiply, soft_add, soft_divide, soft_multiply, soft_subtract,
    subtract,
};
use crate::isa::extension::float::flags::INEXACT;
use crate::isa::extension::float::host::{product, quotient, sum};
use crate::isa::extension::float::{Format, RoundingMode};

const D: Format = Format::DOUBLE;
const NEAREST: RoundingMode = RoundingMode::NearestEven;
const S: Format = Format::SINGLE;

type Operation = fn(Format, u64, u64, RoundingMode, &mut u8) -> u64;

type Path = fn(Format, u64, u64, RoundingMode, &mut u8) -> Option<u64>;

fn operand(state: &mut u64, format: Format) -> u64 {
    let raw = random(state);
    let fraction = match raw >> 60 {
        0 => 0,
        1 => format.fraction(),
        2 => (raw >> 8) & format.fraction() & !0xfff,
        _ => (raw >> 8) & format.fraction(),
    };
    let highest = format.highest_exponent();
    let field = match (raw >> 56) & 15 {
        0 => 0,
        1 => highest,
        2 => 1 + (raw >> 4) % 3,
        3 => highest - 1 - (raw >> 4) % 3,
        4 | 5 => (raw >> 4) % (highest + 1),
        _ => format.bias() as u64 + (raw >> 4) % 60 - 30,
    };
    let sign = if raw & 1 == 1 { format.sign() } else { 0 };
    sign | field << format.fraction_bits | fraction
}

fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn subtracted(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> Option<u64> {
    sum(format, first, second ^ format.sign(), mode, flags)
}

#[test]
fn the_host_path_gives_the_bits_and_flags_of_the_software_path() {
    let operations: [(&str, Operation, Operation, Path); 4] = [
        ("add", add, soft_add, sum),
        ("subtract", subtract, soft_subtract, subtracted),
        ("multiply", multiply, soft_multiply, product),
        ("divide", divide, soft_divide, quotient),
    ];
    let mut state = 0x853c_49e6_748f_ea9b;
    for format in [D, S] {
        for (name, whole, software, path) in operations {
            let (mut taken, mut inexact) = (0, 0);
            for _ in 0..60_000 {
                let (first, second) =
                    (operand(&mut state, format), operand(&mut state, format));
                let (mut ours, mut theirs, mut scratch) = (0, 0, 0);
                let result = whole(format, first, second, NEAREST, &mut ours);
                let expected =
                    software(format, first, second, NEAREST, &mut theirs);
                assert_eq!(
                    (result, ours),
                    (expected, theirs),
                    "{name} {first:#x} {second:#x}"
                );
                if path(format, first, second, NEAREST, &mut scratch).is_some()
                {
                    taken += 1;
                    inexact += u32::from(ours == INEXACT);
                    assert_eq!(ours & !INEXACT, 0);
                }
            }
            assert!(taken > 20_000, "{name}: {taken}");
            assert!(
                inexact > 5_000 && inexact < taken,
                "{name}: {inexact} of {taken}"
            );
        }
    }
}

#[test]
fn the_host_path_leaves_what_it_cannot_decide_to_the_software() {
    let one = 1.0_f64.to_bits();
    let least = f64::MIN_POSITIVE.to_bits();
    let greatest = f64::MAX.to_bits();
    let mut flags = 0;
    for mode in [
        RoundingMode::Down,
        RoundingMode::NearestAway,
        RoundingMode::TowardZero,
        RoundingMode::Up,
    ] {
        assert_eq!(sum(D, one, one, mode, &mut flags), None);
        assert_eq!(product(D, one, one, mode, &mut flags), None);
        assert_eq!(quotient(D, one, one, mode, &mut flags), None);
    }
    for special in [0, 1, D.infinity(false), D.canonical_nan(), D.zero(true)] {
        assert_eq!(sum(D, one, special, NEAREST, &mut flags), None);
        assert_eq!(sum(D, special, one, NEAREST, &mut flags), None);
        assert_eq!(product(D, special, one, NEAREST, &mut flags), None);
        assert_eq!(quotient(D, one, special, NEAREST, &mut flags), None);
    }
    assert_eq!(sum(D, one, one ^ D.sign(), NEAREST, &mut flags), None);
    assert_eq!(sum(D, greatest, greatest, NEAREST, &mut flags), None);
    assert_eq!(product(D, greatest, greatest, NEAREST, &mut flags), None);
    assert_eq!(product(D, least, least, NEAREST, &mut flags), None);
    assert_eq!(quotient(D, least, greatest, NEAREST, &mut flags), None);
    assert_eq!(quotient(D, greatest, least, NEAREST, &mut flags), None);
    assert_eq!(product(D, least, one, NEAREST, &mut flags), None);
    let other = Format {
        exponent_bits: 5,
        fraction_bits: 10,
    };
    assert_eq!(sum(other, 0x3c00, 0x3c00, NEAREST, &mut flags), None);
    assert_eq!(flags, 0);

    assert_eq!(
        sum(D, one, one, NEAREST, &mut flags),
        Some(2.0_f64.to_bits())
    );
    assert_eq!(flags, 0);
    let third = quotient(D, one, 3.0_f64.to_bits(), NEAREST, &mut flags);
    assert_eq!(third, Some((1.0_f64 / 3.0).to_bits()));
    assert_eq!(flags, INEXACT);
    let single = quotient(
        S,
        1.0_f32.to_bits().into(),
        4.0_f32.to_bits().into(),
        NEAREST,
        &mut 0,
    );
    assert_eq!(single, Some(0.25_f32.to_bits().into()));
}
