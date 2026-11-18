// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::support::{VM, application, assert_snapshot, stderr, stdout};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

const HANDLED: &[&str] =
    &["--trap-policy", "vector", "--completion", "0x80100000"];
const TYPED: &[&str] = &[
    "--trap-policy",
    "vector",
    "--completion",
    "0x80100000",
    "--uart",
    "in=/dev/stdin",
];

struct Example {
    folder: &'static str,
    input: &'static str,
    name: &'static str,
    options: &'static [&'static str],
    profile: &'static str,
}

const fn plain(
    folder: &'static str,
    profile: &'static str,
    name: &'static str,
) -> Example {
    Example {
        folder,
        input: "",
        name,
        options: &[],
        profile,
    }
}

const fn completed(profile: &'static str, name: &'static str) -> Example {
    Example {
        folder: "cheriot",
        input: "",
        name,
        options: &["--completion", "0x80100000"],
        profile,
    }
}

const fn handled(
    folder: &'static str,
    profile: &'static str,
    name: &'static str,
) -> Example {
    Example {
        folder,
        input: "",
        name,
        options: HANDLED,
        profile,
    }
}

const fn stopped(name: &'static str) -> Example {
    Example {
        folder: "cheriot",
        input: "",
        name,
        options: &["--trap-policy", "stop"],
        profile: "cheriot",
    }
}

const fn typed(
    folder: &'static str,
    profile: &'static str,
    name: &'static str,
    input: &'static str,
) -> Example {
    Example {
        folder,
        input,
        name,
        options: TYPED,
        profile,
    }
}

const EXAMPLES: &[Example] = &[
    plain("cva6-cheri", "cva6-cheri", "hello-uart"),
    handled("cva6-cheri", "cva6-cheri", "bounds"),
    handled("cva6-cheri", "cva6-cheri", "permissions"),
    plain("cva6-cheri", "cva6-cheri", "levels"),
    handled("cva6-cheri", "cva6-cheri", "trap-return"),
    handled("cva6-cheri", "cva6-cheri", "supervisor"),
    stopped("address"),
    completed("cheriot", "bounds"),
    stopped("bounds-exact"),
    completed("cheriot", "capability-memory"),
    stopped("compare"),
    stopped("compressed"),
    Example {
        folder: "cheriot",
        input: "hiq",
        name: "echo-uart",
        options: &["--trap-policy", "stop", "--uart", "in=/dev/stdin"],
        profile: "cheriot",
    },
    stopped("hello-uart"),
    stopped("inspect"),
    stopped("load-attenuation"),
    completed("cheriot", "pcc"),
    stopped("peripherals"),
    completed("cheriot", "permissions"),
    stopped("representable"),
    stopped("revocation"),
    completed("cheriot-hwr", "revoker"),
    stopped("roots"),
    completed("cheriot", "seal"),
    stopped("sentry"),
    completed("cheriot", "special"),
    stopped("stack-high-water"),
    stopped("store-local"),
    stopped("tag"),
    completed("cheriot", "timer-interrupt"),
    completed("cheriot", "trap-return"),
    handled("cva6", "cva6", "accessed-dirty"),
    handled("cva6", "cva6", "atomics"),
    plain("cva6", "cva6", "bit-manipulation"),
    plain("cva6", "cva6", "compressed-extra"),
    plain("cva6", "cva6", "conditional"),
    plain("cva6", "cva6", "crypto"),
    handled("cva6", "cva6", "delegation"),
    plain("cva6", "cva6", "float"),
    plain("cva6", "cva6", "float-convert"),
    handled("cva6", "cva6", "float-off"),
    plain("cva6", "cva6", "float-rounding"),
    plain("cva6", "cva6", "hello-uart"),
    handled("cva6", "cva6", "modes"),
    handled("cva6", "cva6", "page-permissions"),
    handled("cva6", "cva6", "paging"),
    handled("cva6", "cva6", "pmp"),
    handled("cva6", "cva6", "sfence"),
    handled("cva6", "cva6", "status-traps"),
    handled("cva6", "cva6", "superpage"),
    handled("cva6", "cva6", "supervisor-interrupt"),
    typed("cva6", "cva6", "uart-interrupt", "k"),
    plain("rv32", "rv32imc", "arithmetic"),
    plain("rv32", "rv32imc", "branches"),
    plain("rv32", "rv32imc", "compressed"),
    plain("rv32", "rv32imc", "compressed-forms"),
    plain("rv32", "rv32imc", "csr"),
    Example {
        folder: "rv32",
        input: "hiq",
        name: "echo-uart",
        options: &["--uart", "in=/dev/stdin"],
        profile: "rv32imc",
    },
    plain("rv32", "rv32imc", "ethernet"),
    plain("rv32", "rv32imc", "fence"),
    plain("rv32", "rv32imc", "gpio"),
    plain("rv32", "rv32imc", "hello-uart"),
    plain("rv32", "rv32imc", "hello-world"),
    plain("rv32", "rv32imc", "i2c-eeprom"),
    plain("rv32", "rv32imc", "i2c-sensor"),
    plain("rv32", "rv32imc", "lcd"),
    plain("rv32", "rv32imc", "memory"),
    plain("rv32", "rv32imc", "multiply-divide"),
    plain("rv32", "rv32imc", "peripherals"),
    handled("rv32", "rv32e", "rv32e"),
    handled("rv32", "rv32imc", "software-interrupt"),
    plain("rv32", "rv32imc", "spi-loopback"),
    plain("rv32", "rv32imc", "storage"),
    handled("rv32", "rv32imc", "timer-interrupt"),
    Example {
        folder: "rv32",
        input: "",
        name: "trap-return",
        options: &["--trap-policy", "vector", "--breakpoint", "0x80000018"],
        profile: "rv32imc",
    },
    handled("rv32", "rv32imc", "traps"),
    typed("rv32", "rv32imc", "uart-interrupt", "k"),
    handled("rv32", "rv32imc", "vectored"),
    plain("rv64", "rv64imac", "arithmetic"),
    plain("rv64", "rv64imac", "atomics"),
    plain("rv64", "rv64imac", "compressed"),
    plain("rv64", "rv64imac", "hello-uart"),
    plain("rv64", "rv64imac", "memory"),
    plain("rv64", "rv64imac", "multiply-divide"),
    handled("rv64", "rv64imac", "timer-interrupt"),
    handled("rv64", "rv64imac", "trap-return"),
];

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/assembly")
}

#[test]
fn every_example_has_its_image_and_is_run_by_these_tests() {
    let mut sources = Vec::new();
    for machine in std::fs::read_dir(folder()).unwrap().flatten() {
        let machine_name = machine.file_name().into_string().unwrap();
        for file in std::fs::read_dir(machine.path()).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|ending| ending == "S") {
                assert!(
                    path.with_extension("bin").is_file(),
                    "{} has no image",
                    path.display()
                );
                let name = path.file_stem().unwrap().to_str().unwrap();
                sources.push(format!("{machine_name}/{name}"));
            }
        }
    }
    sources.sort();
    let mut run: Vec<String> = EXAMPLES
        .iter()
        .map(|example| format!("{}/{}", example.folder, example.name))
        .collect();
    run.sort();
    assert_eq!(sources, run);
}

#[test]
fn every_example_runs_to_its_end_and_prints_what_its_snapshot_holds() {
    let snapshots =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/examples");
    for example in EXAMPLES {
        let name = format!("{}/{}", example.folder, example.name);
        let image = folder().join(format!("{name}.bin"));
        let mut child = application(VM)
            .args(["run", "--profile", example.profile, "--raw"])
            .arg(&image)
            .args(["--address", "0x80000000", "--console-format", "raw", "-q"])
            .args(example.options)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut typed = child.stdin.take().unwrap();
        typed.write_all(example.input.as_bytes()).unwrap();
        drop(typed);
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{name}: {}", stderr(&output));
        assert_snapshot(&snapshots, &format!("{name}.txt"), &stdout(&output));
    }
}

#[test]
#[ignore = "needs the assemblers; set CHERI_VM_TEST_ASSEMBLERS=1, and \
            CHERI_VM_TEST_CHERIOT_TOOLS to the folder of the CHERIoT tools"]
fn every_image_is_what_its_source_builds() {
    if std::env::var_os("CHERI_VM_TEST_ASSEMBLERS").is_none() {
        return;
    }
    let cheriot = std::env::var_os("CHERI_VM_TEST_CHERIOT_TOOLS");
    let work = cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let wide = "rv64imafdc_zicsr_zifencei_zicond_zba_zbb_zbc_zbs_zcb_zknd_zkne\
                _zknh";
    for example in EXAMPLES {
        if example.name == "trap-return" && example.folder == "rv32" {
            continue;
        }
        let tools = match (example.folder, &cheriot) {
            ("cheriot", Some(tools)) => PathBuf::from(tools),
            ("cheriot", None) => continue,
            _ => PathBuf::new(),
        };
        let tool = |name: &str| {
            if tools.as_os_str().is_empty() {
                PathBuf::from(name)
            } else {
                tools.join(name)
            }
        };
        let machine: &[&str] = match (example.folder, example.name) {
            ("rv32", "rv32e") => &["--target=riscv32", "-march=rv32e_zicsr"],
            ("rv32", _) => {
                &["--target=riscv32", "-march=rv32imc_zicsr_zifencei"]
            }
            ("rv64", _) => {
                &["--target=riscv64", "-march=rv64imac_zicsr_zifencei"]
            }
            ("cheriot", _) => &[
                "--target=riscv32-unknown-unknown",
                "-mcpu=cheriot",
                "-mabi=cheriot",
            ],
            _ => &["--target=riscv64"],
        };
        let source = folder().join(example.folder);
        let built = |ending: &str| {
            work.path().join(format!("{}.{ending}", example.name))
        };
        let mut compile = std::process::Command::new(tool("clang"));
        compile.args(machine);
        if machine.len() == 1 {
            compile.arg(format!("-march={wide}"));
        }
        let steps = [
            compile
                .arg("-I")
                .arg(&source)
                .arg("-c")
                .arg(source.join(format!("{}.S", example.name)))
                .arg("-o")
                .arg(built("o"))
                .status(),
            std::process::Command::new(tool("ld.lld"))
                .arg("-Ttext=0x80000000")
                .arg(built("o"))
                .arg("-o")
                .arg(built("elf"))
                .status(),
            std::process::Command::new(tool("llvm-objcopy"))
                .args(["-O", "binary"])
                .arg(built("elf"))
                .arg(built("bin"))
                .status(),
        ];
        for step in steps {
            assert!(step.unwrap().success(), "{}", example.name);
        }
        assert_eq!(
            std::fs::read(built("bin")).unwrap(),
            std::fs::read(source.join(format!("{}.bin", example.name)))
                .unwrap(),
            "{}/{}",
            example.folder,
            example.name
        );
    }
}
