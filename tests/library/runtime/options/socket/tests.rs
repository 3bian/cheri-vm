// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::time::Duration;

use crate::machine::host::websocket::Address;
use crate::runtime::options::ethernet::EthernetOptions;
use crate::runtime::options::gpio::GpioOptions;
use crate::runtime::options::i2c::I2cOptions;
use crate::runtime::options::socket::{Host, SocketOptions, parts};
use crate::runtime::options::spi::SpiOptions;
use crate::runtime::options::uart::UartOptions;

impl<T> Host<T> {
    pub(crate) fn linux(&self) -> &T {
        match self {
            Self::Linux(host) => host,
            Self::WebSocket(_) => panic!("bound to a WebSocket"),
        }
    }
}

fn remote(socket: Option<&SocketOptions>) -> Option<(&str, Duration)> {
    let socket = socket?;
    match &socket.address {
        Address::Remote(url) => Some((url.as_str(), socket.timeout)),
        Address::Served => None,
    }
}

fn served(socket: Option<&SocketOptions>) -> bool {
    socket.is_some_and(|socket| matches!(socket.address, Address::Served))
}

#[test]
fn a_websocket_and_the_key_that_names_a_linux_device_are_not_given_together() {
    for (error, expected) in [
        (
            message(SpiOptions::parse("ws,path=/dev/spidev0.0")),
            "--spi takes path or a WebSocket, not both; a WebSocket carries \
             both directions",
        ),
        (
            message(UartOptions::parse("ws,out=uart.log")),
            "--uart takes in and out or a WebSocket, not both; a WebSocket \
             carries both directions",
        ),
        (
            message(EthernetOptions::parse("ws://h/r,interface=tap0")),
            "--ethernet takes interface or a WebSocket, not both; a \
             WebSocket carries both directions",
        ),
        (
            message(GpioOptions::parse("ws,path=/dev/gpiochip0")),
            "--gpio takes path or a WebSocket, not both; a WebSocket carries \
             both directions",
        ),
        (
            message(I2cOptions::parse("ws,path=/dev/i2c-1")),
            "--i2c takes path or a WebSocket, not both; a WebSocket carries \
             both directions",
        ),
    ] {
        assert!(error.ends_with(expected), "{error}");
    }
}

#[test]
fn a_websocket_is_ws_or_a_plain_address_and_its_values_are_in_range() {
    for value in [
        "ws:",
        "ws://",
        "ws:///desk",
        "ws://:9000/uart",
        "wss://127.0.0.1/desk",
        "ws://user@host/desk",
        "ws,timeout=0",
        "ws://h/r,timeout=60001",
        "ws,timeout=soon",
    ] {
        let error = message(UartOptions::parse(value));
        assert!(!error.is_empty(), "{value}");
    }
    assert!(
        message(UartOptions::parse("wss://h/uart"))
            .ends_with("not wss://h/uart; wss is not supported")
    );
    for value in ["http://127.0.0.1/desk", "out=a,ws", "websocket"] {
        assert!(
            message(UartOptions::parse(value)).contains("KEY=VALUE"),
            "{value}"
        );
    }
    assert!(
        message(UartOptions::parse("ws=1"))
            .contains("unknown --uart setting \"ws\"")
    );
    for value in ["ws,pins=32", "ws,pins=4+4", "ws,pins="] {
        assert!(GpioOptions::parse(value).is_err(), "{value}");
    }
    assert!(message(I2cOptions::parse("ws")).ends_with(
        "--i2c on a WebSocket needs addresses=A+A, one address for each"
    ));
    assert!(SpiOptions::parse("ws,write-only=yes").is_err());
    assert_eq!(
        parts("ws://127.0.0.1:8080/desk").unwrap(),
        ("127.0.0.1:8080".to_owned(), "/desk".to_owned())
    );
    assert_eq!(
        parts("ws://localhost/a/b?c=d").unwrap(),
        ("localhost:80".to_owned(), "/a/b?c=d".to_owned())
    );
    assert_eq!(
        parts("ws://[::1]:9000").unwrap(),
        ("[::1]:9000".to_owned(), "/".to_owned())
    );
}

#[test]
fn a_websocket_without_a_timeout_waits_one_second_and_sends_every_pin() {
    let gpio = GpioOptions::parse("ws").unwrap();
    assert!(served(gpio.host.socket()));
    assert_eq!(gpio.host.socket().unwrap().timeout, Duration::from_secs(1));
    assert_eq!(gpio.pins, None);
}

#[test]
fn every_device_option_takes_both_forms_of_a_websocket() {
    let url = "ws://127.0.0.1:9000/uart";
    let expected = Some((url, Duration::from_millis(250)));
    let common = format!("{url},timeout=250");
    let uart = UartOptions::parse(&common).unwrap();
    assert_eq!(remote(uart.host.socket()), expected);
    let ethernet =
        EthernetOptions::parse(&format!("{common},device=ethernet")).unwrap();
    assert_eq!(remote(ethernet.host.socket()), expected);
    assert_eq!(ethernet.device.as_deref(), Some("ethernet"));
    let gpio = GpioOptions::parse(&format!("{common},pins=5+6+13")).unwrap();
    assert_eq!(remote(gpio.host.socket()), expected);
    assert_eq!(gpio.pins, Some(1 << 5 | 1 << 6 | 1 << 13));
    let i2c =
        I2cOptions::parse(&format!("{common},addresses=0x48+0x68")).unwrap();
    assert_eq!(remote(i2c.host.socket()), expected);
    assert_eq!(i2c.addresses, Some(vec![0x48, 0x68]));
    let spi = SpiOptions::parse(&format!("{common},select=1,write-only=true"))
        .unwrap();
    assert_eq!(remote(spi.host.socket()), expected);
    assert_eq!((spi.select, spi.write_only), (1, true));
    assert!(served(
        UartOptions::parse("ws,device=uart1").unwrap().host.socket()
    ));
    assert!(served(EthernetOptions::parse("ws").unwrap().host.socket()));
    assert!(served(
        I2cOptions::parse("ws,addresses=0x48")
            .unwrap()
            .host
            .socket()
    ));
    assert!(served(
        SpiOptions::parse("ws,write-only=true")
            .unwrap()
            .host
            .socket()
    ));
}

#[test]
fn keys_that_only_a_linux_device_has_are_refused_with_a_websocket() {
    for (error, expected) in [
        (
            message(UartOptions::parse("ws,baud=115200")),
            "--uart baud applies to in and out, not to a WebSocket",
        ),
        (
            message(SpiOptions::parse("ws://h/r,speed=1000")),
            "--spi speed applies to path, not to a WebSocket",
        ),
        (
            message(GpioOptions::parse("ws,lines=1+2")),
            "--gpio lines applies to path, not to a WebSocket",
        ),
        (
            message(GpioOptions::parse("ws,bias=pull-up")),
            "--gpio bias applies to path, not to a WebSocket",
        ),
    ] {
        assert!(error.ends_with(expected), "{error}");
    }
}

#[test]
fn keys_that_only_a_websocket_has_are_refused_without_one() {
    for (error, expected) in [
        (
            message(SpiOptions::parse("path=/dev/spidev0.0,timeout=5")),
            "--spi timeout needs a WebSocket: ws or ws://HOST:PORT/PATH",
        ),
        (
            message(GpioOptions::parse("path=/dev/gpiochip0,lines=1,pins=1")),
            "--gpio pins needs a WebSocket: ws or ws://HOST:PORT/PATH",
        ),
    ] {
        assert!(error.ends_with(expected), "{error}");
    }
}

fn message<T>(parsed: Result<T, crate::machine::EmulatorError>) -> String {
    parsed.err().unwrap().to_string()
}

#[test]
fn write_only_is_taken_for_a_linux_bus_as_well() {
    let spi = SpiOptions::parse("path=/dev/spidev0.0,write-only=true").unwrap();
    assert!(spi.write_only);
    assert_eq!(spi.host.linux().speed, 500_000);
}
