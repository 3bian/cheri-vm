// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::support::{VM, application};

#[cfg(all(unix, feature = "gdb"))]
mod gdb_server {
    use crate::support::{Server, VM, application, stderr, stdout};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    fn packet(stream: &mut TcpStream, request: &str) -> String {
        let checksum = request.bytes().fold(0u8, |sum, b| sum.wrapping_add(b));
        write!(stream, "${request}#{checksum:02x}").unwrap();
        receive(stream)
    }

    fn connect(arguments: &[&str]) -> (Server, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let mut child = Server(
            application(VM)
                .arg("debug")
                .args(arguments)
                .args(["--gdb-port", &port.to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        let stream = loop {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                break s;
            };
            assert!(
                Instant::now() < deadline,
                "debug server did not start; status {:?}",
                child.0.try_wait()
            );
            thread::sleep(Duration::from_millis(20));
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        (child, stream)
    }

    fn monitor(stream: &mut TcpStream, command: &str) -> String {
        let encoded: String =
            command.bytes().map(|byte| format!("{byte:02x}")).collect();
        let mut reply = packet(stream, &format!("qRcmd,{encoded}"));
        let mut text = Vec::new();
        while let Some(console) =
            reply.strip_prefix('O').filter(|rest| *rest != "K")
        {
            for pair in console.as_bytes().chunks(2) {
                text.push(
                    u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                        .unwrap(),
                );
            }
            reply = receive(stream);
        }
        assert_eq!(reply, "OK");
        String::from_utf8(text).unwrap()
    }

    fn receive(stream: &mut TcpStream) -> String {
        let mut data = Vec::new();
        let mut b = [0u8; 1];
        loop {
            stream.read_exact(&mut b).unwrap();
            data.push(b[0]);
            if let Some(i) = data.iter().position(|c| *c == b'#')
                && data.len() >= i + 3
            {
                break;
            }
        }
        stream.write_all(b"+").unwrap();
        let start = data.iter().position(|c| *c == b'$').unwrap() + 1;
        let end = data.iter().position(|c| *c == b'#').unwrap();
        String::from_utf8(data[start..end].to_vec()).unwrap()
    }

    #[test]
    fn gdb_rsp_reads_registers_and_memory_sets_breakpoints_and_steps() {
        let (_child, mut stream) = connect(&[
            "tests/fixtures/cheriot/hello_world.elf",
            "--profile",
            "cheriot",
        ]);
        assert!(packet(&mut stream, "?").starts_with('T'));
        assert!(!packet(&mut stream, "g").is_empty());
        assert_eq!(packet(&mut stream, "m80000000,4"), "5b07d003");
        assert_eq!(packet(&mut stream, "Z0,80000000,4"), "OK");
        assert_eq!(packet(&mut stream, "z0,80000000,4"), "OK");
        assert!(packet(&mut stream, "s").starts_with('S'));
    }

    #[test]
    fn gdb_rsp_serves_each_64_bit_machine_at_its_width() {
        for (profile, registers, features) in [
            ("rv64imac", 52, vec!["riscv.cpu", "riscv.csr"]),
            (
                "cva6",
                101,
                vec!["riscv.cpu", "riscv.fpu", "riscv.csr", "riscv.virtual"],
            ),
            (
                "cva6-cheri",
                101,
                vec!["riscv.cpu", "riscv.fpu", "riscv.csr", "riscv.virtual"],
            ),
        ] {
            let cheri = profile == "cva6-cheri";
            let directory =
                cheri_vm::program::temporary::TemporaryDirectory::new()
                    .unwrap();
            let derived = directory.path().join("derive.bin");
            let words =
                [0x0000_0097_u32, 0x0410_0113, 0x0000_0013, 0x0010_0073];
            let bytes: Vec<u8> =
                words.iter().flat_map(|word| word.to_le_bytes()).collect();
            std::fs::write(&derived, bytes).unwrap();
            let derived = derived.to_string_lossy();
            let program = if cheri {
                derived.as_ref()
            } else {
                "examples/assembly/rv32/hello-uart.bin"
            };
            let (_child, mut stream) = connect(&[
                "--raw",
                program,
                "--address",
                "0x80000000",
                "--profile",
                profile,
            ]);
            assert!(packet(&mut stream, "?").starts_with('T'), "{profile}");
            let mut description = String::new();
            loop {
                let request = format!(
                    "qXfer:features:read:target.xml:{:x},400",
                    description.len()
                );
                let part = packet(&mut stream, &request);
                description.push_str(&part[1..]);
                if part.starts_with('l') {
                    break;
                }
            }
            assert!(
                description.contains("<architecture>riscv:rv64</architecture>")
            );
            for feature in &features {
                assert!(description.contains(feature), "{profile}: {feature}");
            }
            assert_eq!(
                description.matches("<reg ").count(),
                registers,
                "{profile}"
            );
            let narrow = description.matches("bitsize=\"32\"").count();
            let all = packet(&mut stream, "g");
            assert_eq!(
                all.len(),
                (registers * 8 - narrow * 4) * 2,
                "{profile}"
            );
            assert_eq!(&all[32 * 16..33 * 16], "0000008000000000", "{profile}");
            let (first, link) = if cheri {
                ("97000000", "0000008000000000")
            } else {
                ("b7000010", "0000001000000000")
            };
            assert_eq!(packet(&mut stream, "m80000000,4"), first, "{profile}");
            assert_eq!(packet(&mut stream, "Z0,80000008,4"), "OK", "{profile}");
            assert_eq!(&packet(&mut stream, "c")[1..3], "05", "{profile}");
            assert_eq!(
                packet(&mut stream, "p20"),
                "0800008000000000",
                "{profile}"
            );
            assert_eq!(packet(&mut stream, "p1"), link, "{profile}");
            assert_eq!(
                packet(&mut stream, "p2"),
                "4100000000000000",
                "{profile}"
            );
            assert_eq!(packet(&mut stream, "z0,80000008,4"), "OK", "{profile}");
            assert_eq!(&packet(&mut stream, "s")[1..3], "05", "{profile}");
            assert_eq!(
                packet(&mut stream, "p20"),
                "0c00008000000000",
                "{profile}"
            );
            assert!(
                packet(&mut stream, "P1=0000000000000000").starts_with('E')
            );
            assert!(packet(&mut stream, "M80000000,1:00").starts_with('E'));
            let status = monitor(&mut stream, "status");
            assert!(status.contains("steps=3"), "{profile}: {status}");
            let capability = monitor(&mut stream, "capability x1");
            if cheri {
                assert!(
                    capability.starts_with(
                        "x1: metadata=0xf01fe80000000000 0x0000000080000000 \
                         (tag=1 "
                    ),
                    "{capability}"
                );
                let integer = monitor(&mut stream, "capability x2");
                assert!(
                    integer.contains("0x0000000000000041 (tag=0 "),
                    "{integer}"
                );
                let program_counter = monitor(&mut stream, "capability pcc");
                assert!(
                    program_counter.contains("0x000000008000000c (tag=1 "),
                    "{program_counter}"
                );
                assert!(
                    monitor(&mut stream, "tags 0x80000000 1")
                        .ends_with(": 0\n")
                );
            } else {
                assert!(
                    capability.starts_with("error: unknown monitor"),
                    "{capability}"
                );
            }
        }
    }

    #[test]
    fn debug_launches_the_gdb_client_with_separate_arguments_after_listening() {
        use std::os::unix::fs::PermissionsExt;
        let dir =
            cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
        let script = dir.path().join("fake gdb");
        let record = dir.path().join("arguments.json");
        std::fs::write(
            &script,
            r##"#!/usr/bin/env python3
import json, os, socket, sys
with open(os.environ['CHERI_VM_TEST_GDB_ARGS'], 'w') as f:
    json.dump(sys.argv[1:], f)
endpoint=sys.argv[-1].split(' ')[-1]
host,port=endpoint.rsplit(':',1)
s=socket.create_connection((host,int(port)),timeout=5)
s.settimeout(5)
s.sendall(b'$D#44')
data=b''
while b'#' not in data or len(data)<data.index(b'#')+3:
    data += s.recv(4096)
s.sendall(b'+')
s.close()
"##,
        )
        .unwrap();
        let mut mode = std::fs::metadata(&script).unwrap().permissions();
        mode.set_mode(0o755);
        std::fs::set_permissions(&script, mode).unwrap();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);
        let out = application(VM)
            .args([
                "debug",
                "tests/fixtures/cheriot/hello_world.elf",
                "--profile",
                "cheriot",
                "--gdb-port",
                &port.to_string(),
                "--gdb",
            ])
            .arg(&script)
            .env("CHERI_VM_TEST_GDB_ARGS", &record)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
        let args: Vec<String> =
            serde_json::from_slice(&std::fs::read(record).unwrap()).unwrap();
        assert_eq!(args[0], "tests/fixtures/cheriot/hello_world.elf");
        assert_eq!(args[1], "-ex");
        assert_eq!(args[2], format!("target remote 127.0.0.1:{port}"));
        assert!(stdout(&out).contains("Hello world"));
    }
}

struct Dap {
    child: std::process::Child,
    input: std::process::ChildStdin,
    output: std::io::BufReader<std::process::ChildStdout>,
    sequence: u64,
}

impl Dap {
    fn event(&mut self, name: &str, console: &mut String) -> serde_json::Value {
        self.until(console, |m| m["type"] == "event" && m["event"] == name)
    }

    fn receive(&mut self) -> serde_json::Value {
        use std::io::{BufRead, Read};
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(
                self.output.read_line(&mut line).unwrap() > 0,
                "adapter closed"
            );
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length:") {
                length = value.trim().parse().unwrap();
            }
        }
        let mut body = vec![0; length];
        self.output.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn response(
        &mut self,
        command: &str,
        console: &mut String,
    ) -> serde_json::Value {
        self.until(console, |m| {
            m["type"] == "response" && m["command"] == command
        })
    }

    fn send(&mut self, command: &str, arguments: serde_json::Value) {
        use std::io::Write;
        self.sequence += 1;
        let mut request = serde_json::json!({
            "seq": self.sequence, "type": "request", "command": command,
        });
        request["arguments"] = arguments;
        let body = request.to_string();
        write!(self.input, "Content-Length: {}\r\n\r\n{body}", body.len())
            .unwrap();
        self.input.flush().unwrap();
    }

    fn start() -> Self {
        Self::start_with(&[])
    }

    fn start_with(arguments: &[&str]) -> Self {
        let mut child = application(VM)
            .args(["debug", "--dap"])
            .args(arguments)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        Self {
            input: child.stdin.take().unwrap(),
            output: std::io::BufReader::new(child.stdout.take().unwrap()),
            child,
            sequence: 0,
        }
    }

    fn until(
        &mut self,
        console: &mut String,
        wanted: impl Fn(&serde_json::Value) -> bool,
    ) -> serde_json::Value {
        loop {
            let message = self.receive();
            if message["event"] == "output" {
                console.push_str(message["body"]["output"].as_str().unwrap());
            } else if wanted(&message) {
                return message;
            } else if message["event"] == "terminated" {
                panic!(
                    "the guest finished while the test was waiting for \
                     something else"
                );
            }
        }
    }
}

#[test]
fn debug_adapter_launches_breaks_steps_and_runs_to_completion() {
    let mut dap = Dap::start();
    let mut console = String::new();
    dap.send("initialize", serde_json::json!({"adapterID": "cheriot"}));
    let capabilities = dap.response("initialize", &mut console);
    assert_eq!(capabilities["body"]["supportsInstructionBreakpoints"], true);

    dap.send(
        "launch",
        serde_json::json!({
            "program": "tests/fixtures/cheriot/hello_world.elf",
            "profile": "cheriot",
        }),
    );
    let launched = dap.response("launch", &mut console);
    assert_eq!(launched["success"], true, "{launched}");
    dap.event("initialized", &mut console);
    dap.send("configurationDone", serde_json::json!({}));
    dap.response("configurationDone", &mut console);
    assert_eq!(
        dap.event("stopped", &mut console)["body"]["reason"],
        "entry"
    );

    dap.send("stackTrace", serde_json::json!({"threadId": 1}));
    let frames = dap.response("stackTrace", &mut console);
    let entry = frames["body"]["stackFrames"][0]["instructionPointerReference"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(entry, "0x80000000");
    dap.send("variables", serde_json::json!({"variablesReference": 1}));
    let variables = dap.response("variables", &mut console);
    assert_eq!(variables["body"]["variables"][0]["name"], "pc");
    assert_eq!(variables["body"]["variables"][0]["value"], entry.as_str());
    dap.send(
        "readMemory",
        serde_json::json!({"memoryReference": entry, "count": 4}),
    );
    assert_eq!(dap.response("readMemory", &mut console)["success"], true);

    dap.send(
        "next",
        serde_json::json!({"threadId": 1, "granularity": "instruction"}),
    );
    dap.response("next", &mut console);
    assert_eq!(dap.event("stopped", &mut console)["body"]["reason"], "step");
    dap.send("evaluate", serde_json::json!({"expression": "pc"}));
    let pc = dap.response("evaluate", &mut console);
    assert_ne!(pc["body"]["result"], entry.as_str());

    let elf = std::fs::read("tests/fixtures/cheriot/hello_world.elf").unwrap();
    let say_hello = cheri_vm::machine::elf::Elf::parse(elf.as_slice())
        .unwrap()
        .symbols()
        .unwrap()
        .unwrap()
        .iter()
        .find(|symbol| symbol.name == "_Z9say_hellov")
        .map(|symbol| format!("{:#010x}", symbol.value))
        .unwrap();
    dap.send(
        "setInstructionBreakpoints",
        serde_json::json!({
            "breakpoints": [{"instructionReference": say_hello}],
        }),
    );
    let set = dap.response("setInstructionBreakpoints", &mut console);
    assert_eq!(set["body"]["breakpoints"][0]["verified"], true);
    dap.send("continue", serde_json::json!({"threadId": 1}));
    dap.response("continue", &mut console);
    assert_eq!(
        dap.event("stopped", &mut console)["body"]["reason"],
        "breakpoint"
    );
    dap.send("stackTrace", serde_json::json!({"threadId": 1}));
    let frames = dap.response("stackTrace", &mut console);
    assert_eq!(frames["body"]["stackFrames"][0]["name"], "_Z9say_hellov");
    dap.send(
        "setInstructionBreakpoints",
        serde_json::json!({"breakpoints": []}),
    );
    dap.response("setInstructionBreakpoints", &mut console);
    dap.send("continue", serde_json::json!({"threadId": 1}));
    dap.response("continue", &mut console);
    assert_eq!(dap.event("exited", &mut console)["body"]["exitCode"], 0);
    dap.event("terminated", &mut console);
    assert!(console.contains("Hello world"), "{console}");

    dap.send("disconnect", serde_json::json!({}));
    dap.response("disconnect", &mut console);
    assert!(dap.child.wait().unwrap().success());
}

#[test]
fn debug_adapter_serves_the_64_bit_and_capability_machines() {
    let directory =
        cheri_vm::program::temporary::TemporaryDirectory::new().unwrap();
    let derived = directory.path().join("derive.bin");
    let words = [0x0000_0097_u32, 0x0410_0113, 0x0000_0013, 0x0010_0073];
    let bytes: Vec<u8> =
        words.iter().flat_map(|word| word.to_le_bytes()).collect();
    std::fs::write(&derived, bytes).unwrap();
    let derived = derived.to_string_lossy();
    let hello = "examples/assembly/rv32/hello-uart.bin";
    for (profile, program, link) in [
        ("rv32imc", hello, "0x10000000"),
        ("rv64imac", hello, "0x0000000010000000"),
        ("cva6", hello, "0x0000000010000000"),
        ("cva6-cheri", derived.as_ref(), "0x0000000080000000"),
    ] {
        let wide = profile != "rv32imc";
        let address = |low: u32| {
            if wide {
                format!("{low:#018x}")
            } else {
                format!("{low:#010x}")
            }
        };
        let mut dap = Dap::start_with(&[
            "--raw",
            program,
            "--address",
            "0x80000000",
            "--profile",
            profile,
        ]);
        let mut console = String::new();
        dap.send("initialize", serde_json::json!({"adapterID": "cheriot"}));
        dap.response("initialize", &mut console);
        dap.send("launch", serde_json::json!({}));
        let launched = dap.response("launch", &mut console);
        assert_eq!(launched["success"], true, "{profile}: {launched}");
        dap.event("initialized", &mut console);
        dap.send("configurationDone", serde_json::json!({}));
        dap.response("configurationDone", &mut console);
        assert_eq!(
            dap.event("stopped", &mut console)["body"]["reason"],
            "entry"
        );
        dap.send("threads", serde_json::json!({}));
        let threads = dap.response("threads", &mut console);
        assert_eq!(threads["body"]["threads"][0]["name"], "RISC-V core");
        dap.send("stackTrace", serde_json::json!({"threadId": 1}));
        let frames = dap.response("stackTrace", &mut console);
        assert_eq!(
            frames["body"]["stackFrames"][0]["instructionPointerReference"],
            address(0x8000_0000).as_str(),
            "{profile}"
        );
        dap.send(
            "setInstructionBreakpoints",
            serde_json::json!({
                "breakpoints": [{"instructionReference": address(0x8000_0008)}],
            }),
        );
        let set = dap.response("setInstructionBreakpoints", &mut console);
        assert_eq!(set["body"]["breakpoints"][0]["verified"], true);
        dap.send("continue", serde_json::json!({"threadId": 1}));
        dap.response("continue", &mut console);
        assert_eq!(
            dap.event("stopped", &mut console)["body"]["reason"],
            "breakpoint",
            "{profile}"
        );
        dap.send("variables", serde_json::json!({"variablesReference": 1}));
        let variables = dap.response("variables", &mut console);
        let program_counter = variable(&variables, "pc")["value"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            program_counter.starts_with(&address(0x8000_0008)),
            "{profile}: {program_counter}"
        );
        let held = variable(&variables, "ra");
        assert_eq!(held["memoryReference"], link, "{profile}");
        let shown = held["value"].as_str().unwrap();
        if profile == "cva6-cheri" {
            assert!(
                shown.starts_with(
                    "0x0000000080000000 (tag=1 base=0x0000000000000000 "
                ),
                "{shown}"
            );
            assert!(program_counter.contains("(tag=1 "), "{program_counter}");
        } else {
            assert_eq!(shown, link, "{profile}");
        }
        assert_eq!(
            variables["body"]["variables"]
                .as_array()
                .unwrap()
                .iter()
                .any(|variable| variable["name"] == "f31"),
            profile.starts_with("cva6"),
            "{profile}"
        );
        dap.send("evaluate", serde_json::json!({"expression": "sp"}));
        let evaluated = dap.response("evaluate", &mut console);
        assert_eq!(evaluated["body"]["result"], address(0x41).as_str());
        dap.send(
            "evaluate",
            serde_json::json!({"expression": "capability pcc"}),
        );
        let capability = dap.response("evaluate", &mut console);
        assert_eq!(
            capability["success"],
            profile == "cva6-cheri",
            "{capability}"
        );
        dap.send(
            "readMemory",
            serde_json::json!({
                "memoryReference": address(0x8000_0000),
                "count": 4,
            }),
        );
        let memory = dap.response("readMemory", &mut console);
        assert_eq!(memory["body"]["address"], address(0x8000_0000).as_str());
        dap.send(
            "next",
            serde_json::json!({"threadId": 1, "granularity": "instruction"}),
        );
        dap.response("next", &mut console);
        assert_eq!(
            dap.event("stopped", &mut console)["body"]["reason"],
            "step"
        );
        dap.send("continue", serde_json::json!({"threadId": 1}));
        dap.response("continue", &mut console);
        let stopped = dap.event("stopped", &mut console);
        assert_eq!(stopped["body"]["reason"], "exception", "{profile}");
        assert_eq!(
            stopped["body"]["description"],
            format!("ebreak at {} (cause 3)", address(0x8000_000c)).as_str()
        );
        dap.send("disconnect", serde_json::json!({}));
        dap.response("disconnect", &mut console);
        assert!(dap.child.wait().unwrap().success());
    }
}

fn variable(variables: &serde_json::Value, name: &str) -> serde_json::Value {
    variables["body"]["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variable| variable["name"] == name)
        .unwrap_or_else(|| panic!("no variable {name} in {variables}"))
        .clone()
}
