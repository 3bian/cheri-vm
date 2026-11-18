// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::privilege::{Mode, PrivilegeModel};
use crate::hart::{Cva6, Hart, StepOutcome};
use crate::machine::TimingPolicy;
use crate::machine::completion::CompletionPolicy;
use crate::machine::devices::uart::Uart;

const PROFILE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/cva6.toml"));

#[test]
fn a_call_from_user_mode_reaches_the_supervisor_which_returns_to_the_user() {
    let mut hart = machine(&[
        0xfff0_0293,
        0x3b02_9073,
        0x01f0_0293,
        0x3a02_9073,
        0x1000_0293,
        0x3022_9073,
        0x0000_0297,
        0x0282_8293,
        0x1052_9073,
        0x0000_0297,
        0x0102_8293,
        0x3412_9073,
        0x3020_0073,
        0x0070_0513,
        0x0000_0073,
        0x0010_0073,
        0x1420_25f3,
        0x1410_2673,
        0x0046_0613,
        0x1416_1073,
        0x0010_0693,
        0x1020_0073,
    ]);
    assert_eq!(next_trap(&mut hart), (0x8000_0038, 8));
    assert_eq!(hart.processor.privilege.mode(), Mode::Supervisor);
    assert_eq!(hart.processor.program_counter, 0x8000_0040);
    assert_eq!(next_trap(&mut hart), (0x8000_003c, 3));
    assert_eq!(hart.processor.privilege.mode(), Mode::Machine);
    assert_eq!(hart.processor.read_register(10).unwrap(), 7);
    assert_eq!(hart.processor.read_register(11).unwrap(), 8);
    assert_eq!(hart.processor.read_register(13).unwrap(), 1);
    assert_eq!(
        hart.processor.machine_registers.machine_status & (3 << 11),
        0
    );
}

#[test]
fn floating_point_computes_boxes_a_single_and_marks_its_state_dirty() {
    let mut hart = machine(&[
        0x0000_22b7,
        0x3002_a073,
        0x0010_0513,
        0x03e5_1513,
        0xf205_0553,
        0x02a5_75d3,
        0xe205_85d3,
        0x0030_0613,
        0xd206_06d3,
        0x0010_0693,
        0xd206_8753,
        0x1ad7_77d3,
        0x0010_2773,
        0x4017_f853,
        0xe008_07d3,
        0xe208_0853,
        0x3000_28f3,
        0x0010_0073,
    ]);
    assert_eq!(next_trap(&mut hart), (0x8000_0044, 3));
    let register = |index| hart.processor.read_register(index).unwrap();
    assert_eq!(register(11), 0x4010_0000_0000_0000);
    assert_eq!(register(14), 1);
    assert_eq!(register(15), 0x3eaa_aaab);
    assert_eq!(register(16), 0xffff_ffff_3eaa_aaab);
    assert_eq!(register(17) >> 63, 1);
    assert_eq!(register(17) & (3 << 13), 3 << 13);
    assert_eq!(hart.processor.float_control, 1);
    assert_ne!(hart.processor.machine_registers.isa & 0x28, 0);
}

#[test]
fn floating_point_is_an_illegal_instruction_until_its_state_is_turned_on() {
    let mut hart = machine(&[0xf205_0553]);
    assert_eq!(next_trap(&mut hart), (0x8000_0000, 2));
    let mut hart = machine(&[0x0010_2773]);
    assert_eq!(next_trap(&mut hart), (0x8000_0000, 2));
}

#[test]
#[ignore = "opt-in: set LINUX_FIRMWARE to the fw_payload.bin that \
            tests/fixtures/linux/build.sh makes"]
fn linux_runs_a_shell_with_a_pipeline_a_background_job_and_floating_point() {
    let firmware =
        std::fs::read(std::env::var("LINUX_FIRMWARE").unwrap()).unwrap();
    let mut configuration = crate::profile::reader::read(PROFILE).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    let mut hart = configuration
        .build_supervisor_raw(0x8000_0000, &firmware)
        .unwrap();
    let mut printed = Vec::new();
    let mut scratch = Vec::new();
    let mut commands = [
        "pi\n",
        "uname -srm\n",
        "seq 1 100 | grep 7 | wc -l\n",
        "sleep 1 & jobs; wait; echo waited $?\n",
        "(sleep 1; echo late) & echo early; wait\n",
        "",
    ]
    .into_iter();
    let mut prompts = 0;
    'run: for _ in 0..2_000_000 {
        for _ in 0..1024 {
            hart.step().unwrap();
        }
        hart.bus
            .drain_output_each(&mut scratch, |_, _, bytes| {
                printed.extend_from_slice(bytes);
                Ok(())
            })
            .unwrap();
        let seen = printed.windows(2).filter(|pair| pair == b"# ").count();
        if seen > prompts {
            prompts = seen;
            match commands.next() {
                Some("") | None => break 'run,
                Some(command) => hart
                    .bus
                    .device_mut::<Uart>("uart")
                    .unwrap()
                    .inject_receive(command.as_bytes())
                    .unwrap(),
            }
        }
    }
    let text = String::from_utf8_lossy(&printed);
    for expected in [
        "Linux version 6.12.0",
        "riscv: base ISA extensions acdfim",
        "riscv-plic: interrupt-controller@c000000: mapped 30 interrupts",
        "ttyS0 at MMIO 0x10000000 (irq = 1",
        "Run /init as init process",
        "cheri-vm init",
        "BusyBox v1.36.1",
        "# pi\r\npi 3.141592653590\r\nroot 1.414213562373\r\n",
        "# uname -srm\r\nLinux 6.12.0 riscv64\r\n",
        "| wc -l\r\n19\r\n",
        "[1]+  Running                    sleep 1\r\nwaited 0\r\n",
        "wait\r\nearly\r\nlate\r\n",
    ] {
        assert!(text.contains(expected), "no {expected} in:\n{text}");
    }
    println!(
        "{} instructions retired",
        hart.processor.machine_registers.retired
    );
}

fn machine(words: &[u32]) -> Hart<Cva6> {
    let firmware: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut configuration = crate::profile::reader::read(PROFILE).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    configuration
        .build_supervisor_raw(0x8000_0000, &firmware)
        .unwrap()
}

pub(crate) fn machine_from_elf(firmware: &[u8]) -> Hart<Cva6> {
    let mut configuration = crate::profile::reader::read(PROFILE).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    configuration.completion = Some(CompletionPolicy::ElfSymbol);
    configuration.build_supervisor_elf(firmware).unwrap()
}

fn next_trap(hart: &mut Hart<Cva6>) -> (u64, u32) {
    for _ in 0..100 {
        if let StepOutcome::Trapped { address, cause, .. } =
            hart.step().unwrap()
        {
            return (address, cause);
        }
    }
    panic!("no trap was taken");
}

#[test]
#[ignore = "opt-in: set OPENSBI_FIRMWARE to an OpenSBI fw_jump.bin built for \
            profiles/cva6.dts"]
fn opensbi_prints_its_banner_and_names_the_platform() {
    let firmware =
        std::fs::read(std::env::var("OPENSBI_FIRMWARE").unwrap()).unwrap();
    let mut configuration = crate::profile::reader::read(PROFILE).unwrap();
    configuration.timing = TimingPolicy::PerStep;
    let mut hart = configuration
        .build_supervisor_raw(0x8000_0000, &firmware)
        .unwrap();
    let mut printed = Vec::new();
    let mut scratch = Vec::new();
    for _ in 0..20_000 {
        for _ in 0..1024 {
            hart.step().unwrap();
        }
        hart.bus
            .drain_output_each(&mut scratch, |_, _, bytes| {
                printed.extend_from_slice(bytes);
                Ok(())
            })
            .unwrap();
        if printed
            .windows(17)
            .any(|window| window == b"Boot HART MEDELEG")
        {
            break;
        }
    }
    let text = String::from_utf8_lossy(&printed);
    for expected in [
        "OpenSBI v",
        "cheri-vm,cva6",
        "aclint-mtimer",
        "uart8250",
        "Boot HART PMP Count         : 8",
        "Boot HART MIDELEG           : 0x0000000000000222",
    ] {
        assert!(text.contains(expected), "no {expected} in:\n{text}");
    }
}

pub(crate) fn run_upstream(variable: &str, expected: usize, skipped: &[&str]) {
    let directory = std::env::var(variable).unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "elf")
        })
        .filter(|path| {
            let name = path.file_stem().unwrap().to_string_lossy();
            !skipped.contains(&name.as_ref())
        })
        .collect();
    paths.sort();
    assert_eq!(
        paths.len(),
        expected,
        "the suite must contain {expected} programs"
    );
    let mut failures = Vec::new();

    for path in &paths {
        let mut hart = machine_from_elf(&std::fs::read(path).unwrap());
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let mut outcome = "timed out".to_owned();
        for _ in 0..2_000_000 {
            match hart.advance() {
                Ok(None) => (),
                Ok(Some(0)) => {
                    outcome.clear();
                    break;
                }
                Ok(Some(code)) => {
                    outcome = format!("failed upstream case {}", code);
                    break;
                }
                Err(error) => {
                    outcome = error.to_string();
                    break;
                }
            }
        }
        if outcome.is_empty() {
            println!("PASS {name}");
        } else {
            failures.push(format!("{name}: {outcome}"));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!("{} upstream programs passed", paths.len());
}

#[test]
fn the_identity_register_names_supervisor_and_user_modes() {
    let hart = machine(&[0x0000_0013]);
    let isa = hart.processor.machine_registers.isa;
    assert_ne!(isa & (1 << 18), 0);
    assert_ne!(isa & (1 << 20), 0);
    assert_eq!(isa >> 62, 2);
}

#[test]
#[ignore = "opt-in: set RISCV64_FLOAT_TEST_BINARIES to a directory of compiled \
            rv64uf and rv64ud riscv-tests"]
fn the_upstream_floating_point_programs_pass() {
    run_upstream("RISCV64_FLOAT_TEST_BINARIES", 23, &[]);
}

#[test]
#[ignore = "opt-in: set RISCV64_PRIVILEGED_TEST_BINARIES to a directory of \
            compiled rv64mi and rv64si riscv-tests"]
fn the_upstream_machine_and_supervisor_programs_pass() {
    run_upstream("RISCV64_PRIVILEGED_TEST_BINARIES", 24, &[]);
}

#[test]
#[ignore = "opt-in: set RISCV64_VIRTUAL_TEST_BINARIES to a directory of \
            riscv-tests compiled for the virtual-memory environment"]
fn the_upstream_unprivileged_programs_pass_in_user_mode_under_sv39() {
    run_upstream("RISCV64_VIRTUAL_TEST_BINARIES", 109, &[]);
}

#[test]
fn user_mode_reaches_no_memory_until_an_entry_opens_it() {
    let mut hart = machine(&[
        0x0000_0297,
        0x0102_8293,
        0x3412_9073,
        0x3020_0073,
        0x0000_0013,
    ]);
    assert_eq!(next_trap(&mut hart), (0x8000_0010, 1));
    assert_eq!(hart.processor.machine_registers.trap_value, 0x8000_0010);
    assert_eq!(hart.processor.privilege.mode(), Mode::Machine);
}
