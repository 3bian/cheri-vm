// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::float::arithmetic::{
    add, divide, fused_multiply_add, multiply, soft_add, soft_divide,
    soft_multiply, soft_subtract, square_root, subtract,
};
use crate::isa::extension::float::compare::{equal, less_or_equal, less_than};
use crate::isa::extension::float::convert::{
    from_integer, resized, to_integer,
};
use crate::isa::extension::float::{Format, RoundingMode};

fn computed(
    function: &str,
    mode: RoundingMode,
    values: &[u64],
    flags: &mut u8,
) -> u64 {
    let (subject, operation) = function.split_once('_').unwrap();
    if let Some(target) = operation.strip_prefix("to_") {
        return match (subject.starts_with('f'), target.starts_with('f')) {
            (true, true) => {
                resized(format(subject), format(target), values[0], mode, flags)
            }
            (true, false) => {
                let (signed, width) = integer(target);
                let result = to_integer(
                    format(subject),
                    values[0],
                    signed,
                    width,
                    mode,
                    flags,
                );
                if width == 32 {
                    result & 0xffff_ffff
                } else {
                    result
                }
            }
            _ => {
                let (signed, width) = integer(subject);
                from_integer(
                    format(target),
                    values[0],
                    signed,
                    width,
                    mode,
                    flags,
                )
            }
        };
    }
    let format = format(subject);
    match operation {
        "add" => add(format, values[0], values[1], mode, flags),
        "sub" => subtract(format, values[0], values[1], mode, flags),
        "mul" => multiply(format, values[0], values[1], mode, flags),
        "div" => divide(format, values[0], values[1], mode, flags),
        "sqrt" => square_root(format, values[0], mode, flags),
        "mulAdd" => fused_multiply_add(
            format,
            [values[0], values[1], values[2]],
            [false, false],
            mode,
            flags,
        ),
        "eq" => u64::from(equal(format, values[0], values[1], flags)),
        "lt" => u64::from(less_than(format, values[0], values[1], flags)),
        "le" => u64::from(less_or_equal(format, values[0], values[1], flags)),
        other => panic!("no operation {other}"),
    }
}

#[test]
#[ignore = "opt-in: set TESTFLOAT_VECTORS to a directory of Berkeley TestFloat \
            vectors"]
fn every_testfloat_vector_gives_the_same_bits_and_flags() {
    let directory = std::env::var("TESTFLOAT_VECTORS").unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 180, "the suite must contain 180 files");
    let mut cases = 0_u64;
    let mut failures = Vec::new();

    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let mut parts = name.split('.');
        let function = parts.next().unwrap();
        let mode = mode(parts.next().unwrap());
        let mut wrong = 0;
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let fields: Vec<u64> = line
                .split(' ')
                .map(|field| u64::from_str_radix(field, 16).unwrap())
                .collect();
            let (values, expected) = fields.split_at(fields.len() - 2);
            let mut flags = 0;
            let result = computed(function, mode, values, &mut flags);
            let mut software_flags = 0;
            let software =
                in_software(function, mode, values, &mut software_flags)
                    .is_none_or(|bits| {
                        (bits, software_flags) == (result, flags)
                    });
            cases += 1;
            if (result, u64::from(flags)) != (expected[0], expected[1])
                || !software
            {
                wrong += 1;
                if wrong == 1 {
                    failures.push(format!(
                        "{name}: {line} gave {result:x} {flags:02x}"
                    ));
                }
            }
        }
        if wrong > 1 {
            failures.push(format!("{name}: {wrong} cases differ"));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!("{cases} vectors match");
}

fn format(name: &str) -> Format {
    match name {
        "f32" => Format::SINGLE,
        "f64" => Format::DOUBLE,
        other => panic!("no format {other}"),
    }
}

fn in_software(
    function: &str,
    mode: RoundingMode,
    values: &[u64],
    flags: &mut u8,
) -> Option<u64> {
    let (subject, operation) = function.split_once('_').unwrap();
    let operation = match operation {
        "add" => soft_add,
        "sub" => soft_subtract,
        "mul" => soft_multiply,
        "div" => soft_divide,
        _ => return None,
    };
    Some(operation(
        format(subject),
        values[0],
        values[1],
        mode,
        flags,
    ))
}

fn integer(name: &str) -> (bool, u32) {
    match name {
        "i32" => (true, 32),
        "ui32" => (false, 32),
        "i64" => (true, 64),
        "ui64" => (false, 64),
        other => panic!("no integer {other}"),
    }
}

fn mode(name: &str) -> RoundingMode {
    match name {
        "near_even" => RoundingMode::NearestEven,
        "minMag" => RoundingMode::TowardZero,
        "min" => RoundingMode::Down,
        "max" => RoundingMode::Up,
        "near_maxMag" => RoundingMode::NearestAway,
        other => panic!("no rounding mode {other}"),
    }
}
