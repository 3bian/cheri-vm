// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::execution::TrapPolicy;
use crate::hart::{Architecture, Hart};
use crate::machine::MachineConfiguration;
use crate::machine::error::{EmulatorError, ExceptionCause};
use std::path::PathBuf;

const EXTENSIONS: &str = "[\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\", \
                          \"zicond\", \"zba\", \"zbb\", \"zbc\", \"zbs\", \
                          \"zcb\", \"zkn\"]";

#[test]
fn a_machine_with_the_bit_extensions_runs_them_and_says_so_in_misa() {
    let profile = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/rv64imac.toml"
    ));
    let listed = "[\"i\", \"m\", \"a\", \"c\", \"zicsr\", \"zifencei\"]";
    let words = [
        0x0050_0113_u32,
        0x0010_0193,
        0x2031_20b3,
        0x3010_2273,
        0x0010_0073,
    ];
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut hart = configuration(profile, listed)
        .build_wide_raw(0x8000_0000, &firmware)
        .unwrap();
    for _ in 0..4 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.read_register(1).unwrap(), 11);
    assert_eq!(
        hart.processor.read_register(4).unwrap(),
        0x8000_0000_0000_1106
    );

    let mut plain = crate::profile::reader::read(
        &profile.replace(listed, "[\"i\", \"zicsr\", \"zba\", \"zbb\"]"),
    )
    .unwrap()
    .build_wide_raw(0x8000_0000, &firmware)
    .unwrap();
    for _ in 0..4 {
        plain.step().unwrap();
    }
    assert_eq!(
        plain.processor.read_register(4).unwrap(),
        0x8000_0000_0000_0100
    );
}

fn configuration(profile: &str, listed: &str) -> MachineConfiguration {
    assert!(profile.contains(listed));
    crate::profile::reader::read(&profile.replace(listed, EXTENSIONS)).unwrap()
}

fn passes<A: Architecture>(
    mut hart: Hart<A>,
    path: &std::path::Path,
    results: impl Fn(&Hart<A>) -> (u64, u64),
) {
    hart.trap_policy = TrapPolicy::Stop;
    for _ in 0..1_000_000 {
        match hart.step() {
            Ok(_) => (),
            Err(EmulatorError::Trap {
                cause: ExceptionCause::Breakpoint,
                ..
            }) => {
                let (code, case) = results(&hart);
                assert_eq!(
                    code,
                    0,
                    "{} failed upstream case {case}",
                    path.display()
                );
                return;
            }
            Err(error) => panic!("{}: {error}", path.display()),
        }
    }
    panic!("{} timed out", path.display());
}

fn programs(variable: &str, count: usize) -> Vec<PathBuf> {
    let directory = std::env::var(variable).unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "elf")
        })
        .collect();
    paths.sort();
    assert_eq!(
        paths.len(),
        count,
        "the suite must contain {count} programs"
    );
    paths
}

#[test]
#[ignore = "opt-in: set RISCV_BIT_TEST_BINARIES to a directory of the compiled \
            rv32 bit-manipulation riscv-tests"]
fn the_upstream_bit_manipulation_programs_pass_at_32_bits() {
    let paths = programs("RISCV_BIT_TEST_BINARIES", 41);
    for path in &paths {
        let hart = configuration(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/profiles/rv32imc.toml"
            )),
            "[\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\"]",
        )
        .build_elf(&std::fs::read(path).unwrap())
        .unwrap();
        passes(hart, path, |hart| {
            (
                hart.processor.read_register(10).unwrap().into(),
                hart.processor.read_register(3).unwrap().into(),
            )
        });
    }
    println!("{} upstream programs passed", paths.len());
}

#[test]
#[ignore = "opt-in: set RISCV64_BIT_TEST_BINARIES to a directory of the \
            compiled rv64 bit-manipulation riscv-tests"]
fn the_upstream_bit_manipulation_programs_pass_at_64_bits() {
    let paths = programs("RISCV64_BIT_TEST_BINARIES", 51);
    for path in &paths {
        let hart = configuration(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/profiles/rv64imac.toml"
            )),
            "[\"i\", \"m\", \"a\", \"c\", \"zicsr\", \"zifencei\"]",
        )
        .build_wide_elf(&std::fs::read(path).unwrap())
        .unwrap();
        passes(hart, path, |hart| {
            (
                hart.processor.read_register(10).unwrap(),
                hart.processor.read_register(3).unwrap(),
            )
        });
    }
    println!("{} upstream programs passed", paths.len());
}
