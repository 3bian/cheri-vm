// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::peer::{Peer, decode, encode, listen, play};
use crate::support::{VM, application, stderr, stdout};
use serde_json::{Value, json};
use std::process::{Child, Output, Stdio};

#[test]
fn a_binary_frame_closes_the_connection_with_1003() {
    let (listener, url) = listen();
    let guest = guest("echo-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    peer.send_binary(b"q");
    assert_eq!(peer.closed_with(), Some(1003));
    assert!(!finished(guest).status.success());
}

#[test]
fn a_message_that_breaks_a_rule_closes_the_connection_with_1008_and_ends_the_run()
 {
    let (listener, url) = listen();
    let guest = guest("echo-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    peer.send(&json!({"type": "uart", "data": "", "speed": 9600}));
    let error = peer.receive().unwrap();
    assert_eq!(error["type"], "error", "{error}");
    assert_eq!(peer.closed_with(), Some(1008));
    let output = finished(guest);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("broke a rule"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_name_with_several_addresses_reaches_the_one_that_listens() {
    let (listener, url) = listen();
    let url = url.replace("127.0.0.1", "localhost");
    let guest = guest("hello-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    let (messages, _) = peer.until_bye();
    assert_eq!(uart_text(&messages), "A");
    assert!(finished(guest).status.success());
}

#[test]
fn a_refused_connection_and_a_refusal_before_ready_name_the_url() {
    let (listener, url) = listen();
    drop(listener);
    let output = finished(guest("echo-uart", &["--uart", &url]));
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!("could not connect to {url}")),
        "{}",
        stderr(&output)
    );
    let (listener, url) = listen();
    let guest = guest("echo-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(
        &json!({"type": "error", "message": "this program takes one machine"}),
    );
    let output = finished(guest);
    assert!(
        stderr(&output).contains(&format!(
            "{url} refused: this program takes one machine; uart has no host"
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_reply_that_never_comes_ends_the_run_with_a_host_error_after_the_timeout() {
    let (listener, url) = listen();
    let guest = guest(
        "i2c-sensor",
        &["--i2c", &format!("{url},addresses=0x48,timeout=200")],
    );
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    assert_eq!(peer.receive().unwrap()["type"], "i2c");
    let output = finished(guest);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "{url} no reply to i2c within 200 ms; i2c/0x48 has no host"
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn addresses_not_given_to_the_websocket_still_reach_the_simulated_peripherals()
{
    let (listener, url) = listen();
    let guest =
        guest("peripherals", &["--i2c", &format!("{url},addresses=0x48")]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    let (messages, _) = peer.until_bye();
    assert!(messages.is_empty(), "{messages:?}");
    let output = finished(guest);
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn bye_says_whether_the_guest_finished_or_was_stopped() {
    let (listener, url) = listen();
    let guest = application(VM)
        .args([
            "run",
            "tests/fixtures/cheriot/hello_world.elf",
            "--profile",
            "cheriot",
            "--uart",
            &url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut peer = Peer::accept(&listener);
    let hello = peer.greet();
    assert_eq!(hello["profile"], "cheriot");
    peer.send(&json!({"type": "ready"}));
    let (messages, bye) = peer.until_bye();
    assert!(uart_text(&messages).contains("Hello world"));
    assert_eq!(
        bye,
        json!({"type": "bye", "reason": "finished", "message": "exit code 0"})
    );
    assert!(finished(guest).status.success());

    let (listener, url) = listen();
    let limited = self::guest("echo-uart", &["--uart", &url, "--steps", "500"]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    let (_, bye) = peer.until_bye();
    assert_eq!(
        bye,
        json!({
            "type": "bye",
            "reason": "stopped",
            "message": "step limit reached",
        })
    );
    assert!(!finished(limited).status.success());
}

fn display(write_only: bool) -> (Vec<Value>, Output) {
    let (listener, url) = listen();
    let spi = format!("{url},write-only={write_only}");
    let gpio = format!("{url},pins=0+1");
    let guest = guest("lcd", &["--spi", &spi, "--gpio", &gpio]);
    let mut first = Peer::accept(&listener);
    let mut second = Peer::accept(&listener);
    let mut hellos = [first.greet(), second.greet()];
    if hellos[0]["kind"] == "spi" {
        std::mem::swap(&mut first, &mut second);
        hellos.swap(0, 1);
    }
    assert_eq!(hellos[0]["device"], "gpio");
    assert_eq!(hellos[1]["device"], "spi/0");
    assert_eq!(hellos[1]["write_only"], write_only);
    for peer in [&mut first, &mut second] {
        peer.send(&json!({"type": "ready"}));
    }
    let messages = play(vec![first, second]);
    (messages, finished(guest))
}

pub(crate) fn drawn(messages: &[Value]) -> Vec<(bool, u8)> {
    let mut data_line = false;
    let mut bytes = Vec::new();
    for message in messages {
        if message["type"] == "gpio" {
            data_line = message["levels"].as_u64().unwrap() & 1 != 0;
        } else if let Some(data) = message.get("data") {
            assert_eq!(message["mode"], 0);
            bytes.extend(
                decode(data.as_str().unwrap())
                    .into_iter()
                    .map(|byte| (data_line, byte)),
            );
        }
    }
    bytes
}

fn finished(guest: Child) -> Output {
    guest.wait_with_output().unwrap()
}

fn guest(image: &str, options: &[&str]) -> Child {
    application(VM)
        .args([
            "run",
            "--raw",
            &format!("examples/assembly/rv32/{image}.bin"),
            "--address",
            "0x80000000",
            "--profile",
            "rv32imc",
        ])
        .args(options)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn no_instruction_runs_before_ready_and_the_wait_is_announced() {
    let (listener, url) = listen();
    let guest = guest("hello-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    std::thread::sleep(std::time::Duration::from_millis(600));
    peer.send(&json!({"type": "ready"}));
    let (messages, _) = peer.until_bye();
    assert_eq!(uart_text(&messages), "A");
    let output = finished(guest);
    assert!(
        stderr(&output).contains(&format!("     Waiting {url}\n")),
        "{}",
        stderr(&output)
    );
}

pub(crate) fn test_pattern() -> Vec<(bool, u8)> {
    let mut expected = vec![
        (false, 0x01),
        (false, 0x11),
        (false, 0x3a),
        (true, 0x05),
        (false, 0x29),
        (false, 0x2a),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 7),
        (false, 0x2b),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 7),
        (false, 0x2c),
    ];
    for color in [[0xf8, 0x00], [0x07, 0xe0], [0x00, 0x1f], [0xff, 0xff]] {
        for _ in 0..16 {
            expected.extend(color.map(|byte| (true, byte)));
        }
    }
    expected
}

#[test]
fn the_display_exchange_gives_the_test_pattern_with_each_byte_answered() {
    let (messages, output) = display(false);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(drawn(&messages), test_pattern());
    assert!(
        messages
            .iter()
            .filter(|message| message.get("data").is_some())
            .all(|message| message["id"].as_u64().is_some())
    );
}

#[test]
fn the_other_end_leaving_ends_the_run_and_names_the_device_left_without_a_host()
{
    let (listener, url) = listen();
    let guest = guest("echo-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    peer.greet();
    peer.send(&json!({"type": "ready"}));
    drop(peer);
    let output = finished(guest);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!("{url} closed; uart has no host")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_sensor_exchange_answers_an_i2c_transaction_with_its_reading() {
    let (listener, url) = listen();
    let guest =
        guest("i2c-sensor", &["--i2c", &format!("{url},addresses=0x48")]);
    let mut peer = Peer::accept(&listener);
    let hello = peer.greet();
    assert_eq!(hello["device"], "i2c/0x48");
    assert_eq!(hello["kind"], "i2c");
    peer.send(&json!({"type": "ready"}));
    let request = peer.receive().unwrap();
    assert_eq!(
        request,
        json!({
            "type": "i2c",
            "id": 1,
            "write": "AA==",
            "read": 2,
        })
    );
    peer.send(
        &json!({"type": "reply", "id": 1, "data": encode(&[0x19, 0x40])}),
    );
    let (_, bye) = peer.until_bye();
    assert_eq!(bye["reason"], "stopped");
    let output = finished(guest);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("1940"), "{}", stdout(&output));
}

#[test]
fn the_terminal_exchange_echoes_through_a_uart_bound_to_a_websocket() {
    let (listener, url) = listen();
    let guest = guest("echo-uart", &["--uart", &url]);
    let mut peer = Peer::accept(&listener);
    let hello = peer.receive().unwrap();
    assert_eq!(
        hello,
        json!({
            "type": "hello",
            "format_version": 1,
            "profile": "rv32imc",
            "device": "uart",
            "kind": "uart",
            "write_only": false,
        })
    );
    peer.send(&json!({"type": "ready"}));
    peer.send(&json!({"type": "uart", "data": encode(b"hiq")}));
    let (messages, bye) = peer.until_bye();
    assert_eq!(uart_text(&messages), "hi");
    assert_eq!(bye["reason"], "stopped", "{bye}");
    assert_eq!(peer.closed_with(), Some(1000));
    let output = finished(guest);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stdout(&output).contains("hi"), "{}", stdout(&output));
}

fn uart_text(messages: &[Value]) -> String {
    let bytes: Vec<u8> = messages
        .iter()
        .filter(|message| message["type"] == "uart")
        .flat_map(|message| decode(message["data"].as_str().unwrap()))
        .collect();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn write_only_sends_the_bytes_between_two_pin_changes_as_one_message_in_order()
{
    let (messages, output) = display(true);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(drawn(&messages), test_pattern());
    let bursts: Vec<usize> = messages
        .iter()
        .filter_map(|message| message.get("data"))
        .map(|data| decode(data.as_str().unwrap()).len())
        .collect();
    assert_eq!(bursts, [3, 1, 2, 4, 1, 4, 1, 128]);
    assert!(messages.iter().all(|message| message.get("id").is_none()));
    assert_eq!(
        messages.first().unwrap(),
        &json!({
            "type": "gpio",
            "levels": 0,
            "direction": 0,
        })
    );
    let selections: Vec<&Value> = messages
        .iter()
        .filter_map(|message| message.get("selected"))
        .collect();
    assert_eq!(selections, [&json!(true), &json!(false)]);
    assert_eq!(messages.last().unwrap()["selected"], false);
}
