// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::support::{
    VM, alone, application, fixture_file, stderr, stdout, vm,
};
use std::io::Write;
use std::process::Stdio;

#[test]
fn a_bad_number_is_reported_with_the_name_of_what_it_was_for() {
    for (option, value, expected) in [
        ("--address", "zz", "raw address is not a number: zz"),
        (
            "--breakpoint",
            "zz",
            "breakpoint address is not a number: zz",
        ),
        (
            "--completion",
            "sometimes",
            "--completion must be disabled, elf-symbol or an address: \
             sometimes",
        ),
        (
            "--memory",
            "address=zz",
            "memory address is not a number: zz",
        ),
        (
            "--memory",
            "address=0,length=zz",
            "memory dump length is not a number: zz",
        ),
        (
            "--uart",
            "out=/dev/null,baud=zz",
            "UART baud rate is not a number: zz",
        ),
        (
            "--spi",
            "path=/dev/null,speed=zz",
            "SPI speed is not a number: zz",
        ),
        (
            "--i2c",
            "path=/dev/null,addresses=zz",
            "I2C address is not a number: zz",
        ),
    ] {
        let mut arguments = vec![
            "run",
            "--profile",
            "profiles/rv32imc.toml",
            "--raw",
            "examples/assembly/rv32/arithmetic.bin",
        ];
        if option != "--address" {
            arguments.extend(["--address", "0x80000000"]);
        }
        arguments.extend([option, value]);
        let refused = vm(&arguments);
        assert!(!refused.status.success(), "{option} {value}");
        assert!(
            stderr(&refused).contains(expected),
            "{option} {value}: {}",
            stderr(&refused)
        );
    }
}

#[test]
fn a_profile_file_runs_and_one_without_a_machine_is_refused() {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let profile = directory.path().join("echo.toml");
    std::fs::write(
        &profile,
        "name = \"echo\"\n[device.uart]\nstart = 0x1000_0000\n",
    )
    .unwrap();
    let elf = std::fs::canonicalize("tests/fixtures/cheriot/hello_world.elf")
        .unwrap();
    let out = application(VM)
        .current_dir(directory.path())
        .arg("run")
        .arg(&elf)
        .args(["--profile", profile.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains(
            "profile echo has no [emulator] table, so cheri-vm cannot run it"
        ),
        "{}",
        stderr(&out)
    );
    assert!(out.stdout.is_empty());
    let emulated = application(VM)
        .current_dir(directory.path())
        .arg("run")
        .arg(&elf)
        .arg("--profile")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("profiles/cheriot.toml"),
        )
        .output()
        .unwrap();
    assert!(emulated.status.success(), "{}", stderr(&emulated));
    assert!(stdout(&emulated).contains("Hello world"));
    let unknown = application(VM)
        .current_dir(directory.path())
        .arg("run")
        .arg(&elf)
        .args(["--profile", "nowhere"])
        .output()
        .unwrap();
    let message = stderr(&unknown);
    assert!(message.contains("unknown profile"), "{message}");
    assert!(message.contains("rv32imc"), "{message}");
}

#[test]
fn every_device_option_takes_its_default_from_its_environment_variable() {
    for (option, variable) in [
        ("--gpio", "CHERI_VM_GPIO"),
        ("--i2c", "CHERI_VM_I2C"),
        ("--spi", "CHERI_VM_SPI"),
        ("--ethernet", "CHERI_VM_ETHERNET"),
        ("--uart", "CHERI_VM_UART"),
    ] {
        let out = application(VM)
            .args([
                "run",
                "tests/fixtures/cheriot/hello_world.elf",
                "--profile",
                "cheriot",
            ])
            .env(variable, "nonsense")
            .output()
            .unwrap();
        assert!(!out.status.success(), "{variable}");
        assert!(
            stderr(&out).contains(option),
            "{variable}: {}",
            stderr(&out)
        );
        let replaced = application(VM)
            .args([
                "run",
                "tests/fixtures/cheriot/hello_world.elf",
                "--profile",
                "cheriot",
                "--steps",
                "1000",
                "--uart",
                "out=/dev/null",
            ])
            .env("CHERI_VM_UART", "nonsense")
            .output()
            .unwrap();
        assert!(
            !stderr(&replaced).contains("--uart takes"),
            "{}",
            stderr(&replaced)
        );
    }
}

#[test]
fn invalid_addresses_and_duplicate_singleton_options_are_rejected() {
    let overflow = vm(&[
        "run",
        "--profile",
        "profile",
        "--raw",
        "firmware",
        "--address",
        "18446744073709551616",
    ]);
    assert!(!overflow.status.success());
    assert!(stderr(&overflow).contains("raw address is not a number"));
    let duplicate = vm(&[
        "run",
        "--profile",
        "a",
        "--profile",
        "b",
        "--raw",
        "firmware",
        "--address",
        "0",
    ]);
    assert!(!duplicate.status.success());
    assert!(stderr(&duplicate).contains("cannot be used multiple times"));
}

#[test]
fn multiple_breakpoints_stop_before_the_first_matching_instruction() {
    let out = vm(&[
        "run",
        "--profile",
        "profiles/rv32imc.toml",
        "--raw",
        "examples/assembly/rv32/arithmetic.bin",
        "--address",
        "0x80000000",
        "--breakpoint",
        "0x8000000c",
        "--breakpoint",
        "0x80000004",
        "--registers",
        "names=x1+x2",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let output = stdout(&out);
    assert!(output.contains("Breakpoint at 0x80000004"), "{output}");
    assert!(output.contains("x01: 0x0000000a"), "{output}");
    assert!(output.contains("x02: 0x00000000"), "{output}");
}

#[cfg(unix)]
#[test]
fn non_utf8_paths_retain_the_application_error() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let path = OsString::from_vec(vec![0xff]);
    let execute = application(VM)
        .args(["run", "--profile", "profiles/rv32imc.toml", "--raw"])
        .arg(&path)
        .args(["--address", "0"])
        .output()
        .unwrap();
    let debug = application(VM)
        .arg("debug")
        .arg(&path)
        .args(["--profile", "profile"])
        .output()
        .unwrap();
    for result in [execute, debug] {
        assert!(!result.status.success());
        assert!(stderr(&result).starts_with("error: path is not UTF-8:"));
    }
}

#[test]
fn quiet_leaves_out_the_line_that_reports_how_the_guest_ended() {
    let run = |extra: &[&str]| {
        let mut arguments = vec![
            "run",
            "tests/fixtures/cheriot/hello_world.elf",
            "--profile",
            "cheriot",
        ];
        arguments.extend(extra);
        let out = vm(&arguments);
        assert!(out.status.success(), "{}", stderr(&out));
        (stdout(&out), stderr(&out))
    };
    let (guest, plain) = run(&[]);
    assert!(plain.contains("Guest exit status: 0"), "{plain}");
    let (quiet_guest, quiet) = run(&["-q"]);
    assert!(quiet.is_empty(), "{quiet}");
    assert_eq!(quiet_guest, guest);
}

fn raw_program(
    words: &[u32],
) -> (cheri_vm::program::temporary::TemporaryDirectory, String) {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let path = directory.path().join("program.bin");
    let bytes: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    std::fs::write(&path, bytes).unwrap();
    let path = path.to_string_lossy().into_owned();
    (directory, path)
}

#[test]
fn registers_names_floating_point_registers_where_the_machine_has_them() {
    let (_directory, program) = raw_program(&[
        0x0000_22b7,
        0x3002_a073,
        0x0030_0313,
        0xd203_00d3,
        0x0010_0073,
    ]);
    let common = ["--raw", &program, "--address", "0x80000000", "--registers"];
    let result = vm(&[
        &["run", "--profile", "cva6"][..],
        &common,
        &["names=f1+f2+fcsr+x6"],
    ]
    .concat());
    assert!(result.status.success(), "{}", stderr(&result));
    let output = stdout(&result);
    assert!(output.contains("x06: 0x0000000000000003 (3)"), "{output}");
    assert!(output.contains("f01: 0x4008000000000000 (3.0)"), "{output}");
    assert!(output.contains("f02: 0x0000000000000000 (0.0)"), "{output}");
    assert!(output.contains("fcsr: 0x00"), "{output}");

    let refused = vm(&[
        &["run", "--profile", "rv64imac"][..],
        &common,
        &["names=f1"],
    ]
    .concat());
    assert!(!refused.status.success());
    let message = stderr(&refused);
    assert!(message.contains("use x0..x31 or pc"), "{message}");
    let named =
        vm(
            &[&["run", "--profile", "cva6"][..], &common, &["names=pcc"]]
                .concat(),
        );
    let message = stderr(&named);
    assert!(
        message.contains("use x0..x31, f0..f31, fcsr or pc"),
        "{message}"
    );
}

#[test]
fn registers_shows_whole_capabilities_on_the_capability_machine() {
    let (_directory, program) = raw_program(&[0x0000_0097, 0x0010_0073]);
    let result = vm(&[
        "run",
        "--profile",
        "cva6-cheri",
        "--raw",
        &program,
        "--address",
        "0x80000000",
        "--registers",
        "names=x1+x2+pcc",
    ]);
    assert!(result.status.success(), "{}", stderr(&result));
    let output = stdout(&result);
    assert!(
        output.contains(
            "x01: 0x0000000080000000 (tag=1 base=0x0000000000000000 \
             top=0x10000000000000000 perms="
        ),
        "{output}"
    );
    assert!(
        output.contains("x02: 0x0000000000000000 (tag=0 "),
        "{output}"
    );
    assert!(
        output.contains("pcc: 0x0000000000000000 (tag=1 "),
        "{output}"
    );
}

#[test]
fn run_needs_a_profile_and_its_help_shows_every_device_form() {
    let help = stdout(&vm(&["run", "--help"]));
    assert!(help.contains("--uart in=PATH|out=PATH"), "{help}");
    assert!(help.contains("path=/dev/gpiochip0,lines=17+27"), "{help}");
    assert!(help.contains("--console-format"));
    assert!(!help.contains("SPEC"));
    let missing = vm(&["run", "firmware.elf"]);
    assert!(!missing.status.success());
    assert!(
        stderr(&missing).contains("no profile selected"),
        "{}",
        stderr(&missing)
    );
}

#[test]
fn run_runs_cheriot_elf_to_guest_completion() {
    let result = vm(&[
        "run",
        "tests/fixtures/cheriot/hello_world.elf",
        "--profile",
        "cheriot",
        "--steps",
        "3000000",
    ]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(stdout(&result).contains("Hello world"));
    assert!(stderr(&result).contains("Guest exit status: 0"));
}

#[test]
fn run_runs_existing_rv32_firmware_and_reports_step_limit() {
    let profile = "profiles/rv32imc.toml";
    let raw = "examples/assembly/rv32/arithmetic.bin";
    let result = vm(&[
        "run",
        "--profile",
        profile,
        "--raw",
        raw,
        "--address",
        "0x80000000",
        "--registers",
        "names=x3+x4",
    ]);
    assert!(result.status.success(), "{}", stderr(&result));
    let output = stdout(&result);
    assert!(output.contains("x03: 0x0000001e"), "{output}");
    assert!(output.contains("x04: 0x00000258"), "{output}");
    let limited = vm(&[
        "run",
        "--profile",
        profile,
        "--raw",
        raw,
        "--address",
        "0x80000000",
        "--steps",
        "1",
    ]);
    assert!(!limited.status.success());
    assert!(stderr(&limited).contains("step limit reached"));
}

#[test]
fn standard_input_reaches_the_uart_when_the_environment_asks_for_it() {
    let mut guest = application(VM)
        .args([
            "run",
            "--raw",
            "examples/assembly/rv32/echo-uart.bin",
            "--address",
            "0x80000000",
            "--profile",
            "rv32imc",
        ])
        .env("CHERI_VM_UART", "in=/dev/stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    guest.stdin.take().unwrap().write_all(b"hiq").unwrap();
    let output = guest.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(stdout(&output).contains("hi"), "{}", stdout(&output));
}

#[test]
fn stats_report_instructions_coverage_and_stacks() {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let report = directory.path().join("stats.json");
    let out = vm(&[
        "run",
        "tests/fixtures/cheriot/hello_world.elf",
        "--profile",
        "cheriot",
        "--stats",
        "-o",
        report.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let stats: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    let total = stats["instructions"].as_u64().unwrap();
    assert!(total > 0);
    let attributed: u64 = stats["compartments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["instructions"].as_u64().unwrap())
        .sum();
    assert_eq!(attributed + stats["outside_code"].as_u64().unwrap(), total);
    for compartment in stats["compartments"].as_array().unwrap() {
        assert!(
            compartment["covered_bytes"].as_u64()
                <= compartment["size"].as_u64()
        );
    }
    assert!(!stats["functions"].as_array().unwrap().is_empty());
    assert!(
        stats["stacks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "thread_stack_1")
    );

    let plain = vm(&[
        "run",
        "tests/fixtures/cheriot/hello_world.elf",
        "--profile",
        "cheriot",
    ]);
    let with_stats = vm(&[
        "run",
        "tests/fixtures/cheriot/hello_world.elf",
        "--profile",
        "cheriot",
        "--stats",
    ]);
    let text = stdout(&with_stats);
    assert!(text.starts_with(&stdout(&plain)), "{text}");
    for section in [
        "Execution\n",
        "Compartments and libraries\n",
        "Stack high-water marks\n",
    ] {
        assert!(text.contains(section), "{text}");
    }
    let raw = vm(&[
        "run",
        "--raw",
        "examples/assembly/rv32/arithmetic.bin",
        "--address",
        "0x80000000",
        "--profile",
        "profiles/rv32imc.toml",
        "--stats",
    ]);
    assert!(!raw.status.success());
    assert!(stderr(&raw).contains("--stats needs an ELF"));
}

#[test]
fn the_64_bit_machine_runs_raw_firmware_and_shows_registers_at_their_width() {
    let result = vm(&[
        "run",
        "--profile",
        "rv64imac",
        "--raw",
        "examples/assembly/rv32/hello-uart.bin",
        "--address",
        "0x80000000",
        "--trace",
        "--registers",
        "names=x1+x2",
    ]);
    assert!(result.status.success(), "{}", stderr(&result));
    let output = format!("{}{}", stdout(&result), stderr(&result));
    assert!(output.contains("UART [uart]: A"), "{output}");
    assert!(
        output.contains("0x0000000080000000 [100000b7/4]"),
        "{output}"
    );
    assert!(output.contains("x01: 0x0000000010000000"), "{output}");
    assert!(output.contains("x02: 0x0000000000000041 (65)"), "{output}");
}

#[test]
fn the_cva6_profile_runs_raw_firmware() {
    let common = [
        "--profile",
        "cva6",
        "--raw",
        "examples/assembly/rv32/hello-uart.bin",
        "--address",
        "0x80000000",
    ];
    let result = vm(&[&["run"][..], &common].concat());
    let output = format!("{}{}", stdout(&result), stderr(&result));
    assert!(output.contains("UART [uart]: A"), "{output}");
    assert!(output.contains("EBREAK at 0x000000008000000c"), "{output}");
}

#[test]
fn a_profile_in_the_settings_folder_runs_by_name_and_extends_a_built_in_one() {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let profiles = directory.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(
        profiles.join("mine.toml"),
        "name = \"mine\"\nextends = \"cheriot\"\n",
    )
    .unwrap();
    let elf = fixture_file("cheriot/hello_world.elf");
    let named = application(VM)
        .env("CHERI_VM_HOME", directory.path())
        .arg("run")
        .arg(&elf)
        .args(["--profile", "mine"])
        .output()
        .unwrap();
    assert!(named.status.success(), "{}", stderr(&named));
    assert!(stdout(&named).contains("Hello world"));
    let from_environment = application(VM)
        .env("CHERI_VM_HOME", directory.path())
        .env("CHERI_VM_PROFILE", "mine")
        .arg("run")
        .arg(&elf)
        .output()
        .unwrap();
    assert!(
        from_environment.status.success(),
        "{}",
        stderr(&from_environment)
    );
    assert!(stdout(&from_environment).contains("Hello world"));
}

#[test]
fn run_with_no_firmware_asks_for_one() {
    let out = vm(&["run", "--profile", "cheriot"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains(
            "name the firmware to run: cheri-vm run FIRMWARE --profile \
             NAME|FILE"
        ),
        "{}",
        stderr(&out)
    );
}

#[test]
fn the_program_copied_alone_into_an_empty_folder_runs_on_its_built_in_profile()
{
    let folder =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let firmware = folder.path().join("hello_world.elf");
    std::fs::copy(fixture_file("cheriot/hello_world.elf"), &firmware).unwrap();
    let output = alone(VM, folder.path())
        .args(["run", "hello_world.elf", "--profile", "cheriot"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("Hello world"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn the_stats_report_takes_the_form_format_or_the_file_name_asks_for() {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let run = |extra: &[&str]| {
        let mut arguments = vec![
            "run",
            "tests/fixtures/cheriot/hello_world.elf",
            "--profile",
            "cheriot",
            "--stats",
        ];
        arguments.extend(extra);
        let out = vm(&arguments);
        assert!(out.status.success(), "{}", stderr(&out));
    };
    let by_name = directory.path().join("stats.toml");
    run(&["-o", by_name.to_str().unwrap()]);
    let table: toml::Table =
        toml::from_str(&std::fs::read_to_string(&by_name).unwrap()).unwrap();
    assert!(table["instructions"].as_integer().unwrap() > 0);
    let chosen = directory.path().join("stats.report");
    run(&["--format", "json", "-o", chosen.to_str().unwrap()]);
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&chosen).unwrap()).unwrap();
    assert_eq!(
        json["instructions"],
        table["instructions"].as_integer().unwrap()
    );
}

#[test]
fn trace_completion_and_trap_policy_are_honored() {
    let base = [
        "run",
        "--profile",
        "profiles/rv32imc.toml",
        "--raw",
        "examples/assembly/rv32/arithmetic.bin",
        "--address",
        "0x80000000",
    ];
    let with = |extra: &[&str]| vm(&[&base[..], extra].concat());

    let traced = with(&["--trace"]);
    assert!(traced.status.success(), "{}", stderr(&traced));
    let trace = stdout(&traced);
    for address in ["0x80000000", "0x80000004", "0x80000008", "0x8000000c"] {
        assert!(
            trace.lines().any(|line| line.starts_with(address)),
            "{trace}"
        );
    }
    let quiet = with(&[]);
    assert!(!stdout(&quiet).contains("0x80000004 ["));

    let stopped = with(&["--trap-policy", "stop"]);
    assert!(stopped.status.success(), "{}", stderr(&stopped));
    assert!(stdout(&stopped).contains("Guest EBREAK at 0x80000010"));
    let vectored = with(&["--trap-policy", "vector", "--steps", "1000"]);
    assert!(!vectored.status.success());
    assert!(
        stderr(&vectored).contains("step limit reached"),
        "{}",
        stderr(&vectored)
    );
    let disabled = with(&["--completion", "disabled"]);
    assert!(disabled.status.success(), "{}", stderr(&disabled));
    let symbol = with(&["--completion", "elf-symbol"]);
    assert!(!symbol.status.success());
    assert!(
        stderr(&symbol).contains("ELF-symbol completion requires ELF firmware"),
        "{}",
        stderr(&symbol)
    );
}
