// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::peer::{Peer, decode, encode, play};
use crate::support::{VM, application, finish};
use crate::websocket::{drawn, test_pattern};
use serde_json::json;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, ChildStderr, Stdio};

struct Run {
    child: Child,
    devices: Vec<String>,
    notices: BufReader<ChildStderr>,
    printed: String,
    served: Option<String>,
}

impl Run {
    fn address(&self) -> &str {
        let url = self.devices.first().unwrap();
        url.strip_prefix("ws://")
            .unwrap()
            .split('/')
            .next()
            .unwrap()
    }

    fn ended(mut self) -> (bool, String) {
        let status = self.child.wait().unwrap();
        let mut rest = String::new();
        self.notices.read_to_string(&mut rest).unwrap();
        (status.success(), format!("{}{rest}", self.printed))
    }

    fn get(&self, target: &str, headers: &str) -> String {
        let host = self.address();
        self.request(&format!(
            "GET {target} HTTP/1.1\r\nHost: {host}\r\n{headers}\r\n"
        ))
    }

    fn request(&self, request: &str) -> String {
        let mut stream = TcpStream::connect(self.address()).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    fn start(image: &str, options: &[&str]) -> Self {
        let mut child = application(VM)
            .args([
                "run",
                "--raw",
                &format!("examples/assembly/rv32/{image}.bin"),
                "--address",
                "0x80000000",
                "--profile",
                "rv32imc",
                "--web-port",
                "0",
            ])
            .args(options)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut run = Self {
            notices: BufReader::new(child.stderr.take().unwrap()),
            child,
            devices: Vec::new(),
            printed: String::new(),
            served: None,
        };
        loop {
            let mut line = String::new();
            assert_ne!(
                run.notices.read_line(&mut line).unwrap(),
                0,
                "{}",
                run.printed
            );
            run.printed.push_str(&line);
            let (verb, object) = line.trim().split_once(' ').unwrap();
            match verb {
                "Serving" => run.served = Some(object.to_owned()),
                "Device" => run.devices.push(object.to_owned()),
                "Waiting" => return run,
                _ => panic!("{line}"),
            }
        }
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        finish(&mut self.child);
    }
}

#[test]
fn a_page_at_the_address_of_a_uart_plays_it_and_the_run_waits_for_the_page() {
    let run = Run::start("echo-uart", &["--uart", "ws"]);
    assert!(run.served.is_none());
    assert_eq!(run.devices.len(), 1);
    assert!(run.devices[0].ends_with("/ws/uart"), "{}", run.devices[0]);
    assert!(
        run.printed
            .ends_with(&format!("     Waiting {}\n", run.devices[0]))
    );
    std::thread::sleep(std::time::Duration::from_millis(300));
    let mut page = Peer::connect(&run.devices[0]);
    assert_eq!(
        page.greet(),
        json!({
            "type": "hello",
            "format_version": 1,
            "profile": "rv32imc",
            "device": "uart",
            "kind": "uart",
            "write_only": false,
        })
    );
    page.send(&json!({"type": "ready"}));
    page.send(&json!({"type": "uart", "data": encode(b"hiq")}));
    let (messages, bye) = page.until_bye();
    let echoed: Vec<u8> = messages
        .iter()
        .flat_map(|message| decode(message["data"].as_str().unwrap()))
        .collect();
    assert_eq!(echoed, b"hi");
    assert_eq!(bye["reason"], "stopped", "{bye}");
    assert_eq!(page.closed_with(), Some(1000));
    let (success, printed) = run.ended();
    assert!(success, "{printed}");
}

#[test]
fn a_page_that_closes_ends_the_run_with_an_error_that_names_the_device() {
    let run = Run::start("echo-uart", &["--uart", "ws"]);
    let mut page = Peer::connect(&run.devices[0]);
    page.greet();
    page.send(&json!({"type": "ready"}));
    drop(page);
    let url = run.devices[0].clone();
    let (success, printed) = run.ended();
    assert!(!success);
    assert!(printed.contains(&format!("{url} closed")), "{printed}");
    assert!(printed.contains("uart has no host"), "{printed}");
}

#[test]
fn a_second_page_is_refused_while_a_device_has_one() {
    let run = Run::start("echo-uart", &["--uart", "ws"]);
    let mut first = Peer::connect(&run.devices[0]);
    first.greet();
    let mut second = Peer::connect(&run.devices[0]);
    assert_eq!(
        second.closed_for(),
        Some((1008, "uart already has a page".to_owned()))
    );
    first.send(&json!({"type": "ready"}));
    first.send(&json!({"type": "uart", "data": encode(b"q")}));
    first.until_bye();
    assert!(run.ended().0);
}

#[test]
fn a_web_socket_from_another_origin_or_for_another_host_is_refused() {
    let run = Run::start("echo-uart", &["--uart", "ws"]);
    let upgrade = "Upgrade: websocket\r\nConnection: Upgrade\r\n\
                   Sec-WebSocket-Version: 13\r\n\
                   Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";
    let elsewhere = run.get(
        "/ws/uart",
        &format!("{upgrade}Origin: http://evil.example\r\n"),
    );
    assert!(
        elsewhere.starts_with("HTTP/1.1 403 Forbidden"),
        "{elsewhere}"
    );
    let named = run.request(&format!(
        "GET /ws/uart HTTP/1.1\r\nHost: evil.example\r\n{upgrade}\r\n"
    ));
    assert!(named.starts_with("HTTP/1.1 403 Forbidden"), "{named}");
    let old = run.get(
        "/ws/uart",
        "Upgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Version: 8\r\n",
    );
    assert!(old.starts_with("HTTP/1.1 426 Upgrade Required"), "{old}");
}

#[test]
fn host_must_be_loopback_and_a_port_in_use_says_how_to_choose_another() {
    let beyond = application(VM)
        .args(["run", "a.elf", "--profile", "rv32imc", "--host", "0.0.0.0"])
        .output()
        .unwrap();
    assert!(!beyond.status.success());
    let message = String::from_utf8_lossy(&beyond.stderr).into_owned();
    assert!(
        message.contains("--host must be a loopback IP address"),
        "{message}"
    );
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port().to_string();
    let busy = application(VM)
        .args([
            "run",
            "--raw",
            "examples/assembly/rv32/echo-uart.bin",
            "--address",
            "0x80000000",
            "--profile",
            "rv32imc",
            "--uart",
            "ws",
            "--web-port",
            &port,
        ])
        .output()
        .unwrap();
    assert!(!busy.status.success());
    let message = String::from_utf8_lossy(&busy.stderr).into_owned();
    assert!(
        message.contains(&format!("could not listen on 127.0.0.1:{port}")),
        "{message}"
    );
    assert!(message.contains("--web-port"), "{message}");
}

#[test]
fn the_display_keeps_its_order_across_the_two_devices_it_is_made_of() {
    let run = Run::start(
        "lcd",
        &["--spi", "ws,write-only=true", "--gpio", "ws,pins=0+1"],
    );
    let address = |name: &str| {
        run.devices
            .iter()
            .find(|url| url.ends_with(name))
            .unwrap()
            .clone()
    };
    let mut pages = vec![
        Peer::connect(&address("/ws/gpio")),
        Peer::connect(&address("/ws/spi/0")),
    ];
    for page in &mut pages {
        page.greet();
        page.send(&json!({"type": "ready"}));
    }
    let messages = play(pages);
    assert_eq!(drawn(&messages), test_pattern());
    let (success, printed) = run.ended();
    assert!(success, "{printed}");
}

#[test]
fn the_web_root_is_served_whole_and_nothing_outside_it() {
    let run = Run::start(
        "echo-uart",
        &["--uart", "ws", "--web-root", "examples/websocket"],
    );
    let root = run.served.clone().unwrap();
    assert_eq!(root, format!("http://{}/", run.address()));
    for (target, kind, holds) in [
        ("/", "text/html; charset=utf-8", "Devices for cheri-vm"),
        ("/index.html", "text/html; charset=utf-8", "terminal.html"),
        ("/css/room.css", "text/css; charset=utf-8", "color-scheme"),
        ("/js/room.js?x=1", "text/javascript; charset=utf-8", "/ws/"),
    ] {
        let answer = run.get(target, "");
        assert!(answer.starts_with("HTTP/1.1 200 OK\r\n"), "{target}");
        assert!(
            answer.contains(&format!("Content-Type: {kind}\r\n")),
            "{target}: {answer}"
        );
        assert!(answer.contains(holds), "{target}");
        assert!(answer.contains("X-Content-Type-Options: nosniff"));
    }
    for target in [
        "/../Cargo.toml",
        "/css/../../Cargo.toml",
        "/.gitignore",
        "/README.md/",
        "/nothing.html",
        "/css",
        "//etc/passwd",
        "/%2e%2e/Cargo.toml",
    ] {
        let answer = run.get(target, "");
        assert!(answer.starts_with("HTTP/1.1 404 Not Found"), "{target}");
    }
    let note = run.get("/ws/uart", "");
    assert!(note.starts_with("HTTP/1.1 426 Upgrade Required"), "{note}");
    assert!(note.contains("<a href=\"/terminal.html\">"), "{note}");
    assert!(note.contains(&run.devices[0]), "{note}");
    let absent = run.get("/ws/uart9", "");
    assert!(absent.starts_with("HTTP/1.1 404 Not Found"), "{absent}");
    assert!(absent.contains("no device at this address"), "{absent}");
    let posted = run.request(&format!(
        "POST / HTTP/1.1\r\nHost: {}\r\nContent-Length: 0\r\n\r\n",
        run.address()
    ));
    assert!(
        posted.starts_with("HTTP/1.1 405 Method Not Allowed"),
        "{posted}"
    );
    let other = run.request("GET / HTTP/1.1\r\nHost: evil.example\r\n\r\n");
    assert!(other.starts_with("HTTP/1.1 403 Forbidden"), "{other}");
    let long = run.get("/", &format!("X-Long: {}\r\n", "a".repeat(9000)));
    assert!(long.starts_with("HTTP/1.1 431 "), "{long}");
    let mut page = Peer::connect(&run.devices[0]);
    page.greet();
    page.send(&json!({"type": "ready"}));
    page.send(&json!({"type": "uart", "data": encode(b"q")}));
    page.until_bye();
    assert!(run.ended().0);
}
