// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::float::arithmetic::{
    add, divide, fused_multiply_add, multiply, square_root, subtract,
};
use crate::isa::extension::float::compare::{
    classify, equal, less_or_equal, less_than, maximum, minimum,
};
use crate::isa::extension::float::convert::{
    from_integer, resized, to_integer,
};
use crate::isa::extension::float::flags::{
    DIVIDE_BY_ZERO, INEXACT, INVALID, OVERFLOW, UNDERFLOW,
};
use crate::isa::extension::float::{Format, RoundingMode};

const D: Format = Format::DOUBLE;
const NEAREST: RoundingMode = RoundingMode::NearestEven;
const S: Format = Format::SINGLE;

#[test]
fn a_conversion_to_an_integer_saturates_and_says_it_was_invalid() {
    let convert = |value: f64, signed, width, mode| {
        let mut flags = 0;
        (
            to_integer(D, double(value), signed, width, mode, &mut flags),
            flags,
        )
    };
    assert_eq!(convert(2.5, true, 32, NEAREST), (2, INEXACT));
    assert_eq!(convert(3.5, true, 32, NEAREST), (4, INEXACT));
    assert_eq!(
        convert(-2.5, true, 32, RoundingMode::TowardZero),
        (-2_i64 as u64, INEXACT)
    );
    assert_eq!(
        convert(-2.5, true, 64, RoundingMode::Down),
        (-3_i64 as u64, INEXACT)
    );
    assert_eq!(convert(3e9, true, 32, NEAREST), (0x7fff_ffff, INVALID));
    assert_eq!(
        convert(-3e9, true, 32, NEAREST),
        (0xffff_ffff_8000_0000, INVALID)
    );
    assert_eq!(convert(3e9, false, 32, NEAREST), (0xffff_ffff_b2d0_5e00, 0));
    assert_eq!(convert(-1.0, false, 32, NEAREST), (0, INVALID));
    assert_eq!(convert(-0.25, false, 64, NEAREST), (0, INEXACT));
    assert_eq!(
        convert(f64::NAN, true, 64, NEAREST),
        (i64::MAX as u64, INVALID)
    );
    assert_eq!(convert(f64::NAN, false, 32, NEAREST), (u64::MAX, INVALID));
    assert_eq!(
        convert(f64::NEG_INFINITY, true, 64, NEAREST),
        (1 << 63, INVALID)
    );
    assert_eq!(
        convert(-9_223_372_036_854_775_808.0, true, 64, NEAREST),
        (1 << 63, 0)
    );
    assert_eq!(
        convert(18_446_744_073_709_551_616.0, false, 64, NEAREST),
        (u64::MAX, INVALID)
    );

    let mut flags = 0;
    assert_eq!(
        from_integer(D, u64::MAX, true, 64, NEAREST, &mut flags),
        double(-1.0)
    );
    assert_eq!(
        from_integer(D, u64::MAX, false, 32, NEAREST, &mut flags),
        double(4_294_967_295.0)
    );
    assert_eq!(flags, 0);
    assert_eq!(
        from_integer(D, u64::MAX, false, 64, NEAREST, &mut flags),
        double(18_446_744_073_709_551_616.0)
    );
    assert_eq!(flags, INEXACT);
}

#[test]
fn a_format_names_its_fields() {
    assert_eq!(S.canonical_nan(), 0x7fc0_0000);
    assert_eq!(D.canonical_nan(), 0x7ff8_0000_0000_0000);
    assert_eq!(S.infinity(true), 0xff80_0000);
    assert_eq!(D.bias(), 1023);
    assert_eq!(RoundingMode::from_bits(4), Some(RoundingMode::NearestAway));
    assert_eq!(RoundingMode::from_bits(5), None);
}

#[test]
fn a_rounding_mode_decides_a_tie_an_overflow_and_the_sign_of_zero() {
    let tie = |mode| {
        let mut flags = 0;
        from_integer(S, (1 << 24) + 1, false, 32, mode, &mut flags)
    };
    assert_eq!(tie(RoundingMode::NearestEven), single(16_777_216.0));
    assert_eq!(tie(RoundingMode::NearestAway), single(16_777_218.0));
    assert_eq!(tie(RoundingMode::TowardZero), single(16_777_216.0));
    assert_eq!(tie(RoundingMode::Up), single(16_777_218.0));
    assert_eq!(tie(RoundingMode::Down), single(16_777_216.0));

    let over = |mode, sign: f64| {
        let mut flags = 0;
        multiply(D, double(sign * f64::MAX), double(2.0), mode, &mut flags)
    };
    assert_eq!(over(RoundingMode::TowardZero, 1.0), double(f64::MAX));
    assert_eq!(over(RoundingMode::Up, 1.0), D.infinity(false));
    assert_eq!(over(RoundingMode::Up, -1.0), double(-f64::MAX));
    assert_eq!(over(RoundingMode::Down, -1.0), D.infinity(true));

    let mut flags = 0;
    assert_eq!(
        subtract(D, double(1.0), double(1.0), NEAREST, &mut flags),
        0
    );
    assert_eq!(
        subtract(D, double(1.0), double(1.0), RoundingMode::Down, &mut flags),
        D.zero(true)
    );
}

fn double(value: f64) -> u64 {
    value.to_bits()
}

#[test]
fn doubles_round_to_nearest_as_the_host_does() {
    let mut seed = 0x1234_5678_9abc_def0;
    for index in 0..200_000 {
        let (a, b, c) = (
            interesting(&mut seed, D),
            interesting(&mut seed, D),
            interesting(&mut seed, D),
        );
        let (x, y, z) =
            (f64::from_bits(a), f64::from_bits(b), f64::from_bits(c));
        let mut flags = 0;
        let case = format!("case {index}: {a:016x} {b:016x} {c:016x}");
        let check = |ours: u64, host: f64| {
            same(D, ours, host.to_bits(), host.is_nan(), &case)
        };
        check(add(D, a, b, NEAREST, &mut flags), x + y);
        check(subtract(D, a, b, NEAREST, &mut flags), x - y);
        check(multiply(D, a, b, NEAREST, &mut flags), x * y);
        check(divide(D, a, b, NEAREST, &mut flags), x / y);
        check(square_root(D, a, NEAREST, &mut flags), x.sqrt());
        check(
            fused_multiply_add(
                D,
                [a, b, c],
                [false, false],
                NEAREST,
                &mut flags,
            ),
            x.mul_add(y, z),
        );
        check(
            fused_multiply_add(D, [a, b, c], [true, true], NEAREST, &mut flags),
            (-x).mul_add(y, -z),
        );
        check(
            resized(S, D, a & 0xffff_ffff, NEAREST, &mut flags),
            f64::from(f32::from_bits(a as u32)),
        );
        assert_eq!(less_than(D, a, b, &mut flags), x < y, "{case}");
        assert_eq!(less_or_equal(D, a, b, &mut flags), x <= y, "{case}");
        assert_eq!(equal(D, a, b, &mut flags), x == y, "{case}");
    }
}

#[test]
fn each_flag_is_raised_by_the_operation_that_earns_it() {
    let flags_of = |run: &dyn Fn(&mut u8) -> u64| {
        let mut flags = 0;
        run(&mut flags);
        flags
    };
    assert_eq!(
        flags_of(&|flags| add(D, double(1.0), double(2.0), NEAREST, flags)),
        0
    );
    assert_eq!(
        flags_of(&|flags| add(D, double(0.1), double(0.2), NEAREST, flags)),
        INEXACT
    );
    assert_eq!(
        flags_of(&|flags| multiply(
            D,
            double(f64::MAX),
            double(2.0),
            NEAREST,
            flags
        )),
        OVERFLOW | INEXACT
    );
    assert_eq!(
        flags_of(&|flags| multiply(
            D,
            double(f64::MIN_POSITIVE),
            double(0.3),
            NEAREST,
            flags
        )),
        UNDERFLOW | INEXACT
    );
    assert_eq!(
        flags_of(&|flags| multiply(
            D,
            double(f64::MIN_POSITIVE),
            double(0.5),
            NEAREST,
            flags
        )),
        0
    );
    assert_eq!(
        flags_of(&|flags| divide(S, single(1.0), single(0.0), NEAREST, flags)),
        DIVIDE_BY_ZERO
    );
    assert_eq!(
        flags_of(&|flags| divide(S, single(0.0), single(0.0), NEAREST, flags)),
        INVALID
    );
    assert_eq!(
        flags_of(&|flags| square_root(S, single(-1.0), NEAREST, flags)),
        INVALID
    );
    assert_eq!(
        flags_of(&|flags| subtract(
            D,
            D.infinity(false),
            D.infinity(false),
            NEAREST,
            flags
        )),
        INVALID
    );
    assert_eq!(
        flags_of(&|flags| add(S, 0x7f80_0001, single(1.0), NEAREST, flags)),
        INVALID
    );
    assert_eq!(
        flags_of(&|flags| add(
            S,
            S.canonical_nan(),
            single(1.0),
            NEAREST,
            flags
        )),
        0
    );
    assert_eq!(
        flags_of(&|flags| fused_multiply_add(
            D,
            [D.infinity(false), 0, D.canonical_nan()],
            [false, false],
            NEAREST,
            flags
        )),
        INVALID
    );
}

#[test]
fn every_class_has_its_bit() {
    for (bits, class) in [
        (S.infinity(true), 0),
        (single(-1.0), 1),
        (0x8000_0001, 2),
        (S.zero(true), 3),
        (0, 4),
        (1, 5),
        (single(1.0), 6),
        (S.infinity(false), 7),
        (0x7f80_0001, 8),
        (S.canonical_nan(), 9),
    ] {
        assert_eq!(classify(S, bits), 1 << class, "{bits:08x}");
    }
    assert_eq!(classify(D, double(-0.0)), 1 << 3);
}

fn interesting(seed: &mut u64, format: Format) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let random = *seed >> 11;
    let bits = format.exponent_bits + format.fraction_bits + 1;
    let all = if bits == 64 {
        u64::MAX
    } else {
        (1 << bits) - 1
    };
    let value = (random ^ (random << 23)) & all;
    match (*seed >> 3) & 15 {
        0 => value & format.sign(),
        1 => format.infinity(value & 1 == 1),
        2 => value & (format.sign() | format.fraction()),
        3 => {
            (value & format.sign())
                | (format.highest_exponent() - 1) << format.fraction_bits
                | 1
        }
        4 => value | format.infinity(false),
        5..=8 => {
            let exponent = (format.bias() as u64)
                .wrapping_add((value >> 4) % 5)
                .wrapping_sub(2);
            (value & (format.sign() | format.fraction()))
                | exponent << format.fraction_bits
        }
        _ => value,
    }
}

fn same(format: Format, ours: u64, host: u64, host_is_nan: bool, case: &str) {
    if host_is_nan {
        assert_eq!(ours, format.canonical_nan(), "{case}");
    } else {
        assert_eq!(ours, host, "{case}");
    }
}

fn single(value: f32) -> u64 {
    u64::from(value.to_bits())
}

#[test]
fn singles_round_to_nearest_as_the_host_does() {
    let mut seed = 0x0fed_cba9_8765_4321;
    for index in 0..200_000 {
        let (a, b, c) = (
            interesting(&mut seed, S),
            interesting(&mut seed, S),
            interesting(&mut seed, S),
        );
        let value = |bits: u64| f32::from_bits(bits as u32);
        let (x, y, z) = (value(a), value(b), value(c));
        let mut flags = 0;
        let case = format!("case {index}: {a:08x} {b:08x} {c:08x}");
        let check = |ours: u64, host: f32| {
            same(S, ours, u64::from(host.to_bits()), host.is_nan(), &case)
        };
        check(add(S, a, b, NEAREST, &mut flags), x + y);
        check(subtract(S, a, b, NEAREST, &mut flags), x - y);
        check(multiply(S, a, b, NEAREST, &mut flags), x * y);
        check(divide(S, a, b, NEAREST, &mut flags), x / y);
        check(square_root(S, a, NEAREST, &mut flags), x.sqrt());
        check(
            fused_multiply_add(
                S,
                [a, b, c],
                [false, false],
                NEAREST,
                &mut flags,
            ),
            x.mul_add(y, z),
        );
        let wide = interesting(&mut seed, D);
        check(
            resized(D, S, wide, NEAREST, &mut flags),
            f64::from_bits(wide) as f32,
        );
    }
}

#[test]
fn the_least_and_greatest_prefer_a_number_to_a_nan_and_order_the_zeros() {
    let mut flags = 0;
    let (positive, negative) = (D.zero(false), D.zero(true));
    assert_eq!(minimum(D, positive, negative, &mut flags), negative);
    assert_eq!(minimum(D, negative, positive, &mut flags), negative);
    assert_eq!(maximum(D, negative, positive, &mut flags), positive);
    assert_eq!(
        minimum(D, D.canonical_nan(), double(3.0), &mut flags),
        double(3.0)
    );
    assert_eq!(
        maximum(D, double(3.0), D.canonical_nan(), &mut flags),
        double(3.0)
    );
    assert_eq!(
        maximum(D, D.canonical_nan(), D.canonical_nan() | 1, &mut flags),
        D.canonical_nan()
    );
    assert_eq!(flags, 0);
    assert_eq!(
        minimum(S, 0x7f80_0001, single(1.0), &mut flags),
        single(1.0)
    );
    assert_eq!(flags, INVALID);
    assert_eq!(
        maximum(S, single(-2.0), single(1.5), &mut flags),
        single(1.5)
    );
    assert_eq!(
        minimum(S, single(-2.0), single(1.5), &mut flags),
        single(-2.0)
    );

    let mut flags = 0;
    assert!(!equal(D, D.canonical_nan(), double(1.0), &mut flags));
    assert_eq!(flags, 0);
    assert!(!less_than(D, D.canonical_nan(), double(1.0), &mut flags));
    assert_eq!(flags, INVALID);
}

#[test]
fn tininess_is_judged_after_rounding() {
    let below = S.infinity(false) >> 8 | 0x007f_ffff;
    let mut flags = 0;
    let half_step = single(f32::from_bits(0x3f80_0001));
    assert_eq!(
        multiply(S, below, half_step, NEAREST, &mut flags),
        0x0080_0000
    );
    assert_eq!(flags, INEXACT);

    let mut flags = 0;
    assert_eq!(
        multiply(S, below, half_step, RoundingMode::TowardZero, &mut flags),
        0x007f_ffff
    );
    assert_eq!(flags, INEXACT | UNDERFLOW);
}
