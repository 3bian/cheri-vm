// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::process::{Child, Command, Output};

pub const VM: &str = "cheri-vm";

pub struct Server(pub Child);

impl Drop for Server {
    fn drop(&mut self) {
        finish(&mut self.0);
    }
}

pub fn alone(name: &str, folder: &std::path::Path) -> Command {
    let copy = folder.join(name);
    std::fs::copy(programs().join(name), &copy).unwrap();
    let mut command = Command::new(copy);
    command
        .current_dir(folder)
        .env("CHERI_VM_HOME", folder.join("home"));
    command
}

pub fn application(name: &str) -> Command {
    let program = programs().join(name);
    assert!(
        program.is_file(),
        "{} is not built; run the tests with cargo test",
        program.display()
    );
    let mut command = Command::new(program);
    command
        .env(
            "CHERI_VM_HOME",
            std::env::temp_dir().join("cheri-vm-tests-no-home"),
        )
        .env_remove("CHERI_VM_PROFILE");
    command
}

#[cfg(unix)]
fn ask(child: &mut Child) {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;
    if let Ok(process) = i32::try_from(child.id()) {
        let _ = kill(Pid::from_raw(process), Signal::SIGTERM);
    }
}

#[cfg(not(unix))]
fn ask(child: &mut Child) {
    let _ = child.kill();
}

pub fn finish(child: &mut Child) -> Option<i32> {
    ask(child);
    let asked = std::time::Instant::now();
    while asked.elapsed() < std::time::Duration::from_secs(5) {
        if let Ok(Some(status)) = child.try_wait() {
            return status.code();
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let _ = child.kill();
    child.wait().ok().and_then(|status| status.code())
}

pub fn fixture_file(name: &str) -> PathBuf {
    workspace().join("tests/fixtures").join(name)
}

pub fn flat(output: &Output) -> String {
    stdout(output)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn programs() -> PathBuf {
    let test = std::env::current_exe().unwrap();
    test.ancestors().nth(2).unwrap().to_path_buf()
}

pub fn repository() -> PathBuf {
    workspace()
}

pub fn run_application(name: &str, arguments: &[&str]) -> Output {
    application(name).args(arguments).output().unwrap()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\n  ", " ")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn vm(arguments: &[&str]) -> Output {
    run_application(VM, arguments)
}

fn workspace() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
