// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::execution::TrapPolicy;
use crate::hart::{Hart, Rv64Plain, StepOutcome};
use crate::machine::TimingPolicy;
use crate::machine::error::{EmulatorError, ExceptionCause};
use crate::machine::firmware::{load_elf, load_wide_elf};

const BASE: u64 = 0x8000_0000;

#[test]
fn a_64_bit_machine_runs_from_memory_above_4_gib_and_a_32_bit_one_refuses_it() {
    let profile = |processor: &str| {
        format!(
            "name = \"high\"\n[emulator]\nprocessor = \
             \"{processor}\"\nextensions = [\"i\", \"zicsr\"]\ntiming = \
             \"per-step\"\nmemory = {{ ram = {{ start = 0x1_0000_0000, length \
             = 0x1000 }} }}\n"
        )
    };
    let firmware: Vec<u8> = [0x0010_0093_u32, 0x0010_0073]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    let mut hart = crate::profile::reader::read(&profile("rv64"))
        .unwrap()
        .build_wide_raw(0x1_0000_0000, &firmware)
        .unwrap();
    hart.trap_policy = TrapPolicy::Stop;
    run_to_the_breakpoint(&mut hart);
    assert_eq!(hart.processor.read_register(1).unwrap(), 1);

    let refusal = crate::profile::reader::read(&profile("rv32"))
        .unwrap()
        .build(0x1_0000_0000)
        .err()
        .unwrap()
        .to_string();
    assert!(refusal.contains("below 4 GiB"), "{refusal}");
}

#[test]
fn a_doubleword_is_stored_and_loaded_whole_and_a_word_load_extends_as_it_says()
{
    let mut hart = machine(&[
        0x0000_0097,
        0x1000_8093,
        0xffe0_0113,
        0x0241_1113,
        0x1231_0113,
        0x0020_b023,
        0x0000_b183,
        0x0040_e203,
        0x0040_a283,
        0x0020_a423,
        0x0080_b303,
        0x0010_0073,
    ]);
    run_to_the_breakpoint(&mut hart);
    assert_eq!(
        registers(&hart, 1..=6),
        [
            BASE + 0x100,
            0xffff_ffe0_0000_0123,
            0xffff_ffe0_0000_0123,
            0xffff_ffe0,
            0xffff_ffff_ffff_ffe0,
            0x123,
        ]
    );
    assert_eq!(hart.bus.read(BASE + 0x100, 4).unwrap(), 0x123);
    assert_eq!(hart.bus.read(BASE + 0x104, 4).unwrap(), 0xffff_ffe0);
}

#[test]
fn a_fetch_above_four_gibibytes_traps_with_the_whole_address() {
    let words = [
        0x0000_0097,
        0x0180_8093,
        0x3050_9073,
        0x0010_0113,
        0x0201_1113,
        0x0001_0067,
        0x3410_21f3,
        0x3420_2273,
        0x3430_22f3,
        0x0010_0073,
    ];
    let mut hart = machine(&words);
    hart.trap_policy = TrapPolicy::Vector;
    for _ in 0..6 {
        assert!(matches!(hart.step(), Ok(StepOutcome::Retired { .. })));
    }
    assert_eq!(hart.processor.program_counter, 0x1_0000_0000);
    assert_eq!(
        hart.step().unwrap(),
        StepOutcome::Trapped {
            address: 0x1_0000_0000,
            cause: 1,
            interrupt: false,
            value: 0x1_0000_0000,
        }
    );
    for _ in 0..3 {
        hart.step().unwrap();
    }
    assert_eq!(registers(&hart, 3..=5), [0x1_0000_0000, 1, 0x1_0000_0000]);

    let mut hart = machine(&words);
    for _ in 0..6 {
        hart.step().unwrap();
    }
    assert!(matches!(
        hart.step(),
        Err(EmulatorError::Trap {
            cause: ExceptionCause::InstructionAccessFault,
            address: 0x1_0000_0000,
            value: 0x1_0000_0000,
        })
    ));
}

#[test]
fn a_store_conditional_needs_its_reservation_and_an_atomic_operation_returns_what_was_there()
 {
    let mut hart = machine(&[
        0x0000_0097,
        0x1000_8093,
        0x0050_0113,
        0x0020_b023,
        0x1000_b1af,
        0x0090_0213,
        0x1840_b2af,
        0x1820_b32f,
        0x0000_b383,
        0xfff0_0413,
        0x0080_a4af,
        0x0000_b503,
        0xe080_b5af,
        0x0000_b603,
        0x8040_a6af,
        0x0000_a703,
        0x0840_b7af,
        0x1000_a82f,
        0x0080_8913,
        0x1829_28af,
        0x0010_0073,
    ]);
    run_to_the_breakpoint(&mut hart);
    assert_eq!(
        registers(&hart, 3..=17),
        [
            5,
            9,
            0,
            1,
            9,
            u64::MAX,
            9,
            8,
            8,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            9,
            1,
        ]
    );
    assert_eq!(hart.bus.read(BASE + 0x100, 4).unwrap(), 9);
    assert_eq!(hart.bus.read(BASE + 0x104, 4).unwrap(), 0);
}

#[test]
fn a_word_instruction_works_on_32_bits_and_sign_extends_the_result() {
    let mut hart = machine(&[
        0xfff0_0093,
        0x0210_d093,
        0x0010_811b,
        0x0010_81bb,
        0x0200_9213,
        0x41f1_529b,
        0x0210_833b,
        0x0211_43bb,
        0x0242_3433,
        0x4010_04bb,
        0x0041_551b,
        0x0211_75bb,
        0x0010_0073,
    ]);
    run_to_the_breakpoint(&mut hart);
    assert_eq!(
        registers(&hart, 1..=11),
        [
            0x7fff_ffff,
            0xffff_ffff_8000_0000,
            0xffff_ffff_ffff_fffe,
            0x7fff_ffff_0000_0000,
            u64::MAX,
            1,
            u64::MAX,
            0x3fff_ffff_0000_0001,
            0xffff_ffff_8000_0001,
            0x0800_0000,
            1,
        ]
    );
}

#[test]
fn each_loader_takes_its_own_class_of_elf_and_names_it() {
    let mut image = vec![0_u8; 124];
    image[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    for (offset, value) in [(16, 2_u16), (18, 243), (52, 64), (54, 56), (56, 1)]
    {
        image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    image[20..24].copy_from_slice(&1_u32.to_le_bytes());
    for (offset, value) in [
        (24, BASE),
        (32, 64),
        (64 + 8, 120),
        (64 + 16, BASE),
        (64 + 24, BASE),
        (64 + 32, 4),
        (64 + 40, 8),
    ] {
        image[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    image[64..68].copy_from_slice(&1_u32.to_le_bytes());
    image[68..72].copy_from_slice(&5_u32.to_le_bytes());
    image[120..124].copy_from_slice(&0x0010_0073_u32.to_le_bytes());

    let mut hart = machine(&[0x0000_0013]);
    let extensions = hart.processor.extensions;
    assert_eq!(
        load_wide_elf(&mut hart.bus, &image, extensions).unwrap(),
        BASE
    );
    assert_eq!(hart.bus.read(BASE, 4).unwrap(), 0x0010_0073);
    assert_eq!(hart.bus.read(BASE + 4, 4).unwrap(), 0);
    let message = load_elf(&mut hart.bus, &image, extensions)
        .unwrap_err()
        .to_string();
    assert!(message.contains("ELF32"), "{message}");

    let mut narrow = vec![0_u8; 132];
    super::write_elf32_header(&mut narrow, BASE as u32, 0, 1);
    let message = load_wide_elf(&mut hart.bus, &narrow, extensions)
        .unwrap_err()
        .to_string();
    assert!(message.contains("ELF64"), "{message}");
}

fn machine(words: &[u32]) -> Hart<Rv64Plain> {
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    machine_from_bytes(&firmware)
}

fn machine_from_bytes(firmware: &[u8]) -> Hart<Rv64Plain> {
    let mut configuration = crate::profile::reader::read(include_str!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/rv64imac.toml")
    ))
    .unwrap();
    configuration.timing = TimingPolicy::PerStep;
    let mut hart = configuration.build_wide_raw(BASE, firmware).unwrap();
    hart.trap_policy = TrapPolicy::Stop;
    hart
}

fn registers(
    hart: &Hart<Rv64Plain>,
    range: std::ops::RangeInclusive<u8>,
) -> Vec<u64> {
    range
        .map(|index| hart.processor.read_register(index).unwrap())
        .collect()
}

fn run_to_the_breakpoint(hart: &mut Hart<Rv64Plain>) {
    for _ in 0..100 {
        match hart.step() {
            Ok(_) => (),
            Err(EmulatorError::Trap {
                cause: ExceptionCause::Breakpoint,
                ..
            }) => return,
            Err(error) => panic!("{error}"),
        }
    }
    panic!("no breakpoint was reached");
}

#[test]
fn the_compressed_forms_are_the_64_bit_ones() {
    let halfwords: [u16; 12] = [
        0x4405, 0x1422, 0x347d, 0x0804, 0xe080, 0x6088, 0xe422, 0x65a2, 0x9d01,
        0x9d21, 0x9005, 0x9002,
    ];
    let firmware: Vec<u8> = halfwords
        .iter()
        .flat_map(|halfword| halfword.to_le_bytes())
        .collect();
    let mut hart = machine_from_bytes(&firmware);
    hart.processor.write_register(2, BASE + 0x200).unwrap();
    for _ in 0..3 {
        hart.step().unwrap();
    }
    assert_eq!(hart.processor.read_register(8).unwrap(), u64::MAX);
    for _ in 0..5 {
        hart.step().unwrap();
    }
    assert_eq!(registers(&hart, 9..=11), [BASE + 0x210, u64::MAX, u64::MAX]);
    hart.step().unwrap();
    assert_eq!(hart.processor.read_register(10).unwrap(), 0);
    run_to_the_breakpoint(&mut hart);
    assert_eq!(hart.processor.read_register(10).unwrap(), u64::MAX);
    assert_eq!(hart.processor.read_register(8).unwrap(), 0x7fff_ffff);
    assert_eq!(hart.processor.program_counter, 0);
}

#[test]
fn the_machine_registers_are_64_bits_and_have_no_high_halves() {
    let mut hart =
        machine(&[0x3010_20f3, 0xb000_2173, 0xb800_21f3, 0x0010_0073]);
    hart.step().unwrap();
    hart.step().unwrap();
    assert_eq!(
        hart.processor.read_register(1).unwrap(),
        0x8000_0000_0000_1105
    );
    assert!(hart.processor.read_register(2).unwrap() < 16);
    assert!(matches!(
        hart.step(),
        Err(EmulatorError::Trap {
            cause: ExceptionCause::IllegalInstruction,
            value: 0xb800_21f3,
            ..
        })
    ));
}

#[test]
#[ignore = "opt-in: set RISCV64_TEST_BINARIES to a directory of compiled \
            64-bit riscv-tests"]
fn the_upstream_64_bit_unprivileged_programs_pass() {
    let directory = std::env::var("RISCV64_TEST_BINARIES").unwrap();
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
        86,
        "the suite must contain all 86 selected programs"
    );

    for path in &paths {
        let mut hart = crate::profile::reader::read(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/profiles/rv64imac.toml"
        )))
        .unwrap()
        .build_wide_elf(&std::fs::read(path).unwrap())
        .unwrap();
        hart.trap_policy = TrapPolicy::Stop;
        let mut completed = false;

        for _ in 0..1_000_000 {
            match hart.step() {
                Ok(_) => (),
                Err(EmulatorError::Trap {
                    cause: ExceptionCause::Breakpoint,
                    ..
                }) => {
                    assert_eq!(
                        hart.processor.read_register(10).unwrap(),
                        0,
                        "{} failed upstream case {}",
                        path.display(),
                        hart.processor.read_register(3).unwrap()
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
            hart.processor.machine_registers.retired
        );
    }

    println!("{} upstream programs passed", paths.len());
}
