// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::execution::TrapPolicy;
use crate::machine::EmulatorError;
use crate::machine::ExceptionCause;
use crate::machine::firmware::load_elf;

#[test]
#[ignore = "opt-in: set RISCV_FLOAT_TEST_BINARIES to a directory of compiled \
            rv32uf and rv32ud riscv-tests"]
fn the_upstream_floating_point_programs_pass_at_32_bits() {
    let directory = std::env::var("RISCV_FLOAT_TEST_BINARIES").unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "elf")
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 21, "the suite must contain 21 programs");
    let profile = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/rv32imc.toml"
    ))
    .replace("\"m\", \"c\"", "\"m\", \"a\", \"f\", \"d\", \"c\"");

    for path in &paths {
        let mut configuration = crate::profile::reader::read(&profile).unwrap();
        configuration.timing = crate::machine::TimingPolicy::PerStep;
        configuration.completion =
            Some(crate::machine::completion::CompletionPolicy::ElfSymbol);
        let mut computer = configuration
            .build_elf(&std::fs::read(path).unwrap())
            .unwrap();
        let code = (0..2_000_000).find_map(|_| computer.advance().unwrap());
        assert_eq!(code, Some(0), "{}", path.display());
    }

    println!("{} upstream programs passed", paths.len());
}

#[test]
#[ignore = "opt-in: set RISCV_TEST_BINARIES to a directory of compiled \
            riscv-tests"]
fn the_upstream_unprivileged_programs_pass() {
    let directory = std::env::var("RISCV_TEST_BINARIES").unwrap();
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
        60,
        "the pinned suite must contain all 60 selected programs"
    );
    let profile = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/rv32imc.toml"
    ));
    let atomic = profile.replace("\"m\", \"c\"", "\"m\", \"a\", \"c\"");

    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy();
        let mut computer =
            crate::profile::reader::read(if name.starts_with("rv32ua") {
                &atomic
            } else {
                profile
            })
            .unwrap()
            .build(0x8000_0000)
            .unwrap();
        computer.trap_policy = TrapPolicy::Stop;
        computer.processor.program_counter = load_elf(
            &mut computer.bus,
            &std::fs::read(path).unwrap(),
            computer.processor.extensions,
        )
        .unwrap();
        let mut completed = false;

        for _ in 0..1_000_000 {
            match computer.step() {
                Ok(_) => (),
                Err(EmulatorError::Trap {
                    cause: ExceptionCause::Breakpoint,
                    ..
                }) => {
                    assert_eq!(
                        computer.processor.read_register(10).unwrap(),
                        0,
                        "{} failed upstream case {}",
                        path.display(),
                        computer.processor.read_register(3).unwrap()
                    );
                    completed = true;
                    break;
                }
                Err(error) => panic!("{}: {error}", path.display()),
            }
        }

        assert!(completed, "{} timed out", path.display());
        println!(
            "PASS {} ({} retired)",
            path.file_name().unwrap().to_string_lossy(),
            computer.processor.machine_registers.retired
        );
    }

    println!("{} upstream programs passed", paths.len());
}
