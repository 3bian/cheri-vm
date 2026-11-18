// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tungstenite::{Message, WebSocket};

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::ethernet::{Ethernet, NetworkBackend};
use crate::machine::devices::gpio::Gpio;
use crate::machine::devices::i2c::{Eeprom, I2c};
use crate::machine::devices::spi::Spi;
use crate::machine::host::websocket::{
    Address, Target, WebSocketBinding, WebSocketConnections,
};
use crate::websocket::base64::{decode, encode};

const ETHERNET: u32 = 0x1000_1000;
const GPIO: u32 = 0x1000_0000;
const I2C: u32 = 0x1000_3000;
const SPI: u32 = 0x1000_2000;

type Socket = WebSocket<TcpStream>;

#[test]
fn a_peer_that_never_answers_a_close_does_not_keep_the_emulator() {
    let (url, peer) = remote(|socket| {
        greet(socket);
        assert_eq!(next(socket)["type"], "gpio");
        tell(socket, &json!({"type": "claim", "endpoints": ["gpio"]}));
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(10)
            && socket.send(Message::text("{}")).is_ok()
        {}
    });
    let mut bus = bus();
    let _connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(&url, "gpio", Target::Gpio { pins: u32::MAX })],
        "test",
        Vec::new(),
    )
    .unwrap();
    let started = Instant::now();
    eventually(&mut bus, |bus| bus.check_host_errors().is_err());
    assert!(started.elapsed() < Duration::from_secs(3));
    drop(bus);
    drop(_connections);
    peer.join().unwrap();
}

#[test]
fn a_reply_nobody_waits_for_and_input_for_an_unbound_endpoint_close_with_1008()
{
    for intrusion in [
        json!({"type": "reply", "id": 9, "data": ""}),
        json!({"type": "uart", "data": ""}),
        json!({"type": "claim", "endpoints": ["gpio"]}),
        json!({"type": "gpio", "levels": 1}),
    ] {
        let sent = intrusion.clone();
        let (connected, wait) = std::sync::mpsc::channel();
        let (url, peer) = remote(move |socket| {
            greet(socket);
            assert_eq!(next(socket)["type"], "gpio");
            wait.recv().unwrap();
            tell(socket, &sent);
            closing(socket)
        });
        let mut bus = bus();
        let _connections = WebSocketConnections::connect(
            &mut bus,
            &[binding(&url, "gpio", Target::Gpio { pins: u32::MAX })],
            "test",
            Vec::new(),
        )
        .unwrap();
        connected.send(()).unwrap();
        eventually(&mut bus, |bus| bus.check_host_errors().is_err());
        let error = bus.check_host_errors().unwrap_err().to_string();
        assert!(error.contains("broke a rule"), "{intrusion}: {error}");
        assert!(error.ends_with("gpio has no host"), "{error}");
        let (seen, code) = peer.join().unwrap();
        assert_eq!(seen.last().unwrap()["type"], "error", "{intrusion}");
        assert_eq!(code, Some(1008), "{intrusion}");
    }
}

#[test]
fn a_transaction_waits_for_its_reply_and_the_wait_is_kept_out_of_guest_time() {
    let (url, peer) = remote(|socket| {
        greet(socket);
        let request = next(socket);
        std::thread::sleep(Duration::from_millis(60));
        tell(
            socket,
            &json!({
                "type": "reply",
                "id": request["id"],
                "data": encode(&[0x19, 0x40]),
            }),
        );
        let mut seen = vec![request];
        seen.extend(until_bye(socket));
        seen
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(
            &url,
            "i2c",
            Target::I2c {
                addresses: vec![0x48],
            },
        )],
        "test",
        Vec::new(),
    )
    .unwrap();
    assert_eq!(bus.take_host_wait(), Duration::ZERO);
    assert_eq!(read_two_bytes(&mut bus, 0x48).unwrap(), [0x19, 0x40]);
    let waited = bus.take_host_wait();
    assert!(waited >= Duration::from_millis(60), "{waited:?}");
    assert_eq!(bus.take_host_wait(), Duration::ZERO);
    bus.write(I2C.into(), 4, 0x50).unwrap();
    bus.write((I2C + 4).into(), 4, 0).unwrap();
    bus.write((I2C + 8).into(), 4, 1).unwrap();
    assert_eq!(bus.read((I2C + 12).into(), 4).unwrap() & 8, 0);
    bus.write((I2C + 8).into(), 4, 3).unwrap();
    connections.finish(Err(&finished()));
    assert_eq!(
        peer.join().unwrap(),
        [json!({
            "type": "i2c",
            "id": 1,
            "write": "Bw==",
            "read": 2,
        })]
    );
}

#[test]
fn a_write_only_burst_is_split_at_4096_bytes_and_another_device_waits_for_it() {
    let (spi_url, spi_peer) = remote(|socket| {
        greet(socket);
        until_bye_in_step(socket)
    });
    let (gpio_url, gpio_peer) = remote(|socket| {
        greet(socket);
        until_bye_in_step(socket)
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[
            binding(
                &spi_url,
                "spi",
                Target::Spi {
                    select: 0,
                    write_only: true,
                },
            ),
            binding(&gpio_url, "gpio", Target::Gpio { pins: u32::MAX }),
        ],
        "test",
        Vec::new(),
    )
    .unwrap();
    bus.write((SPI + 8).into(), 4, 0).unwrap();
    for index in 0..5000_u32 {
        bus.write(SPI.into(), 4, index & 0xff).unwrap();
        assert_eq!(bus.read(SPI.into(), 4).unwrap(), 0xff);
    }
    bus.write((GPIO + 8).into(), 4, 1).unwrap();
    bus.write((SPI + 12).into(), 4, 3).unwrap();
    for byte in [1, 2, 3] {
        bus.write(SPI.into(), 4, byte).unwrap();
    }
    bus.write((SPI + 8).into(), 4, u32::MAX).unwrap();
    connections.finish(Err(&finished()));
    let shape = |seen: &[Value]| -> Vec<String> {
        seen.iter()
            .map(|message| match message.get("data") {
                Some(data) => format!(
                    "{} bytes in mode {}",
                    decode(data.as_str().unwrap()).unwrap().len(),
                    message["mode"]
                ),
                None => message["type"].as_str().unwrap().to_owned(),
            })
            .collect()
    };
    let spi = spi_peer.join().unwrap();
    assert_eq!(
        shape(&spi),
        [
            "spi",
            "4096 bytes in mode 0",
            "904 bytes in mode 0",
            "sync",
            "3 bytes in mode 3",
            "spi"
        ]
    );
    assert_eq!(spi[0]["selected"], true);
    assert_eq!(spi[5]["selected"], false);
    let first = decode(spi[1]["data"].as_str().unwrap()).unwrap();
    assert_eq!(first[..3], [0, 1, 2]);
    assert!(spi.iter().all(
        |message| message["type"] == "sync" || message.get("id").is_none()
    ));
    assert_eq!(
        shape(&gpio_peer.join().unwrap()),
        ["gpio", "sync", "gpio", "sync"]
    );
}

#[test]
fn a_write_only_burst_that_stops_growing_leaves_while_the_chip_is_still_selected()
 {
    let (url, peer) = remote(|socket| {
        greet(socket);
        let select = next(socket);
        let burst = next(socket);
        (select, burst, until_bye(socket))
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(
            &url,
            "spi",
            Target::Spi {
                select: 0,
                write_only: true,
            },
        )],
        "test",
        Vec::new(),
    )
    .unwrap();
    bus.write((SPI + 8).into(), 4, 0).unwrap();
    for byte in [1, 2, 3] {
        bus.write(SPI.into(), 4, byte).unwrap();
    }
    connections.settle().unwrap();
    bus.write(SPI.into(), 4, 4).unwrap();
    connections.settle().unwrap();
    connections.settle().unwrap();
    connections.finish(Err(&finished()));
    let (select, burst, rest) = peer.join().unwrap();
    assert_eq!(select["selected"], true);
    assert_eq!(
        decode(burst["data"].as_str().unwrap()).unwrap(),
        [1, 2, 3, 4]
    );
    assert!(rest.is_empty(), "{rest:?}");
}

fn binding(url: &str, device: &str, target: Target) -> WebSocketBinding {
    WebSocketBinding {
        address: Address::Remote(url.into()),
        device: device.into(),
        target,
        timeout: Duration::from_millis(150),
    }
}

fn bus() -> Bus {
    let mut bus = Bus::new();
    let mut i2c = I2c::new();
    i2c.attach(0x50, Box::new(Eeprom::default())).unwrap();
    bus.add_device("gpio".into(), GPIO.into(), 0x100, Box::new(Gpio::new()))
        .unwrap();
    bus.add_device(
        "ethernet".into(),
        ETHERNET.into(),
        0x100,
        Box::new(Ethernet::new(NetworkBackend::InMemory)),
    )
    .unwrap();
    bus.add_device("spi".into(), SPI.into(), 0x100, Box::new(Spi::new()))
        .unwrap();
    bus.add_device("i2c".into(), I2C.into(), 0x100, Box::new(i2c))
        .unwrap();
    bus
}

fn closing(socket: &mut Socket) -> (Vec<Value>, Option<u16>) {
    let mut seen = Vec::new();
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                seen.push(serde_json::from_str(&text).unwrap())
            }
            Ok(Message::Close(frame)) => {
                return (seen, frame.map(|frame| frame.code.into()));
            }
            Ok(_) => {}
            Err(_) => return (seen, None),
        }
    }
}

#[test]
fn ethernet_frames_cross_in_both_directions() {
    let frame: Vec<u8> = (0..60).collect();
    let sent = frame.clone();
    let (url, peer) = remote(move |socket| {
        greet(socket);
        tell(
            socket,
            &json!({
                "type": "ethernet",
                "frame": encode(&sent),
            }),
        );
        until_bye(socket)
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(&url, "ethernet", Target::Ethernet)],
        "test",
        Vec::new(),
    )
    .unwrap();
    eventually(&mut bus, |bus| {
        bus.read((ETHERNET + 20).into(), 4).unwrap() == 60
    });
    assert_eq!(bus.read((ETHERNET + 24).into(), 4).unwrap(), 0x0302_0100);
    bus.write((ETHERNET + 8).into(), 4, 60).unwrap();
    for word in frame.chunks(4) {
        let value = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        bus.write((ETHERNET + 12).into(), 4, value).unwrap();
    }
    bus.write((ETHERNET + 16).into(), 4, 1).unwrap();
    assert_eq!(bus.read((ETHERNET + 4).into(), 4).unwrap() & 1, 1);
    connections.finish(Err(&finished()));
    assert_eq!(
        peer.join().unwrap(),
        [json!({
            "type": "ethernet",
            "frame": encode(&frame),
        })]
    );
}

fn eventually(bus: &mut Bus, mut reached: impl FnMut(&mut Bus) -> bool) {
    let started = Instant::now();
    while !reached(bus) {
        assert!(started.elapsed() < Duration::from_secs(10), "never reached");
        std::thread::sleep(Duration::from_millis(1));
        bus.poll_host_inputs();
    }
}

fn finished() -> EmulatorError {
    EmulatorError::Execution("the test is over".into())
}

fn greet(socket: &mut Socket) -> Value {
    let hello = next(socket);
    tell(socket, &json!({"type": "ready"}));
    hello
}

#[test]
fn hello_names_the_device_of_the_connection_and_bye_says_why_the_run_ended() {
    let (url, peer) = remote(|socket| {
        let hello = greet(socket);
        let mut rest = until_bye(socket);
        rest.insert(0, hello);
        (rest, closing(socket).1)
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(
            &url,
            "spi",
            Target::Spi {
                select: 2,
                write_only: true,
            },
        )],
        "bench-profile",
        Vec::new(),
    )
    .unwrap();
    connections.finish(Err(&finished()));
    let (seen, code) = peer.join().unwrap();
    assert_eq!(
        seen,
        [json!({
            "type": "hello",
            "format_version": 1,
            "profile": "bench-profile",
            "device": "spi/2",
            "kind": "spi",
            "write_only": true,
        })]
    );
    assert_eq!(code, Some(1000));
}

fn next(socket: &mut Socket) -> Value {
    loop {
        match socket.read().unwrap() {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Close(frame) => panic!("closed: {frame:?}"),
            _ => {}
        }
    }
}

#[test]
fn pin_levels_leave_when_they_change_and_inputs_move_only_bound_input_pins() {
    let (url, peer) = remote(|socket| {
        greet(socket);
        let first = next(socket);
        tell(
            socket,
            &json!({
                "type": "gpio",
                "levels": 0x23,
                "mask": 0x2f,
            }),
        );
        let mut seen = vec![first, next(socket), next(socket)];
        tell(
            socket,
            &json!({
                "type": "gpio",
                "levels": 0x40,
                "mask": 0x61,
            }),
        );
        seen.extend(until_bye(socket));
        seen
    });
    let mut bus = bus();
    let connections = WebSocketConnections::connect(
        &mut bus,
        &[binding(&url, "gpio", Target::Gpio { pins: 0x63 })],
        "test",
        Vec::new(),
    )
    .unwrap();
    eventually(&mut bus, |bus| bus.read(GPIO.into(), 4).unwrap() == 0x23);
    assert_eq!(bus.read((GPIO + 16).into(), 4).unwrap(), 0x23);
    bus.write((GPIO + 8).into(), 4, 0x81).unwrap();
    bus.write((GPIO + 4).into(), 4, 0x81).unwrap();
    bus.write((GPIO + 4).into(), 4, 0x81).unwrap();
    eventually(&mut bus, |bus| bus.read(GPIO.into(), 4).unwrap() == 0xc3);
    connections.finish(Err(&finished()));
    assert_eq!(
        peer.join().unwrap(),
        [
            json!({
                "type": "gpio",
                "levels": 0,
                "direction": 0,
            }),
            json!({
                "type": "gpio",
                "levels": 0,
                "direction": 1,
            }),
            json!({
                "type": "gpio",
                "levels": 1,
                "direction": 1,
            }),
        ]
    );
}

fn read_two_bytes(
    bus: &mut Bus,
    address: u32,
) -> Result<[u32; 2], EmulatorError> {
    bus.write(I2C.into(), 4, address)?;
    bus.write((I2C + 16).into(), 4, 1 << 16)?;
    bus.write((I2C + 4).into(), 4, 0x07)?;
    bus.write((I2C + 8).into(), 4, 1)?;
    bus.write((I2C + 8).into(), 4, 2)?;
    let first = bus.read((I2C + 4).into(), 4)?;
    bus.write((I2C + 8).into(), 4, 2)?;
    let second = bus.read((I2C + 4).into(), 4)?;
    bus.write((I2C + 8).into(), 4, 3)?;
    Ok([first, second])
}

fn remote<T: Send + 'static>(
    script: impl FnOnce(&mut Socket) -> T + Send + 'static,
) -> (String, JoinHandle<T>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("ws://{}/desk", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        script(&mut socket)
    });
    (url, thread)
}

fn tell(socket: &mut Socket, value: &Value) {
    socket.send(Message::text(value.to_string())).unwrap();
}

fn until_bye(socket: &mut Socket) -> Vec<Value> {
    let mut seen = Vec::new();
    loop {
        let message = next(socket);
        if message["type"] == "bye" {
            return seen;
        }
        seen.push(message);
    }
}

fn until_bye_in_step(socket: &mut Socket) -> Vec<Value> {
    let mut seen = Vec::new();
    loop {
        let message = next(socket);
        if message["type"] == "bye" {
            return seen;
        }
        if message["type"] == "sync" {
            tell(
                socket,
                &json!({"type": "reply", "id": message["id"], "data": ""}),
            );
        }
        seen.push(message);
    }
}
