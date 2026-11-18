// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::ethernet::Ethernet;
use crate::machine::devices::ethernet::NetworkBackend;
use crate::machine::devices::gpio::Gpio;
use crate::machine::devices::i2c::I2c;
use crate::machine::devices::spi::Spi;
use crate::machine::host::HostConnections;
use crate::runtime::options::ethernet::EthernetOptions;
use crate::runtime::options::gpio::{GpioChip, GpioOptions};
use crate::runtime::options::host::HostOptions;
use crate::runtime::options::socket::Host;

#[test]
fn a_device_is_bound_to_a_websocket_or_to_linux_and_a_chip_select_once() {
    use crate::runtime::options::spi::SpiOptions;

    let twice = HostOptions {
        gpio: vec![
            GpioOptions::parse("ws,device=gpio0").unwrap(),
            chip(Some("gpio0".into()), vec![1]),
        ],
        ..HostOptions::default()
    };
    let error = crate::runtime::host_bindings::prepare(&bus(1), twice)
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("duplicate GPIO binding for gpio0"),
        "{error}"
    );
    let select = HostOptions {
        spi: vec![
            SpiOptions::parse("ws").unwrap(),
            SpiOptions::parse("ws://h/bench").unwrap(),
        ],
        ..HostOptions::default()
    };
    let error = crate::runtime::host_bindings::prepare(&bus(1), select)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("duplicate SPI device/select"), "{error}");
}

#[test]
fn automatic_device_selection_requires_exactly_one_matching_device() {
    for count in [0, 2] {
        let error =
            prepare(&bus(count), vec![chip(None, vec![17, 27])], vec![])
                .err()
                .unwrap()
                .to_string();
        assert!(
            error.contains(if count == 0 {
                "no GPIO"
            } else {
                "gpio0, gpio1"
            }),
            "{error}"
        );
    }
    let connections = prepare(
        &bus(1),
        vec![chip(None, vec![17, 27])],
        vec![tap(None, "tap0")],
    )
    .unwrap();
    assert_eq!(connections[0].gpio_chip.as_ref().unwrap().0, "gpio0");
    assert_eq!(
        connections[1].ethernet_interface.as_ref().unwrap().0,
        "ethernet0"
    );
    let error = prepare(&bus(2), vec![], vec![tap(None, "tap0")])
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("ethernet0, ethernet1"), "{error}");
}

fn bus(count: u32) -> Bus {
    let mut bus = Bus::new();
    for index in 0..count {
        let base = 0x10000000 + index * 0x10000;
        bus.add_device(
            format!("gpio{index}"),
            base.into(),
            0x1000,
            Box::new(Gpio::new()),
        )
        .unwrap();
        bus.add_device(
            format!("ethernet{index}"),
            (base + 0x1000).into(),
            0x1000,
            Box::new(Ethernet::new(NetworkBackend::InMemory)),
        )
        .unwrap();
    }
    for index in 0..count {
        let base = 0x20000000 + index * 0x10000;
        bus.add_device(
            format!("spi{index}"),
            base.into(),
            256,
            Box::new(Spi::new()),
        )
        .unwrap();
        bus.add_device(
            format!("i2c{index}"),
            (base + 0x1000).into(),
            256,
            Box::new(I2c::new()),
        )
        .unwrap();
    }
    bus
}

fn chip(device: Option<String>, lines: Vec<u32>) -> GpioOptions {
    GpioOptions {
        device,
        host: Host::Linux(GpioChip {
            lines,
            path: "/dev/gpiochip0".into(),
            settings: Default::default(),
        }),
        pins: None,
    }
}

#[test]
fn duplicate_devices_and_overlapping_host_resources_fail_before_attachment() {
    for (first, second, interface) in [
        ("ethernet0", "ethernet0", "tap1"),
        ("ethernet0", "ethernet1", "tap0"),
    ] {
        let error = prepare(
            &bus(2),
            vec![],
            vec![
                tap(Some(first.into()), "tap0"),
                tap(Some(second.into()), interface),
            ],
        )
        .err()
        .unwrap()
        .to_string();
        assert!(
            error.contains(if first == second {
                "duplicate Ethernet binding"
            } else {
                "only be bound once"
            }),
            "{error}"
        );
    }
    let error = prepare(
        &bus(2),
        [0, 1]
            .map(|index| chip(Some(format!("gpio{index}")), vec![17]))
            .into(),
        vec![],
    )
    .err()
    .unwrap()
    .to_string();
    assert!(error.contains("unique across bindings"), "{error}");
}

#[test]
fn explicit_multi_device_bindings_are_matched_by_name_in_any_order() {
    let gpio = [1, 0].map(|index| {
        chip(Some(format!("gpio{index}")), vec![index * 2, index * 2 + 1])
    });
    let taps = [0, 1].map(|index| {
        tap(Some(format!("ethernet{index}")), &format!("tap{index}"))
    });
    let connections = prepare(&bus(2), gpio.into(), taps.into()).unwrap();
    assert_eq!(connections.len(), 4);
    assert_eq!(connections[0].gpio_chip.as_ref().unwrap().0, "gpio1");
    assert_eq!(connections[0].gpio_lines.as_ref().unwrap(), &[2, 3]);
    assert_eq!(connections[1].gpio_chip.as_ref().unwrap().0, "gpio0");
    assert_eq!(
        connections[2].ethernet_interface.as_ref().unwrap().1,
        "tap0"
    );
    assert_eq!(
        connections[3].ethernet_interface.as_ref().unwrap().1,
        "tap1"
    );
}

fn prepare(
    bus: &Bus,
    gpio: Vec<GpioOptions>,
    ethernet: Vec<EthernetOptions>,
) -> Result<Vec<HostConnections>, EmulatorError> {
    crate::runtime::host_bindings::prepare(
        bus,
        HostOptions {
            gpio,
            ethernet,
            ..HostOptions::default()
        },
    )
    .map(|prepared| prepared.linux)
}

#[cfg(target_os = "linux")]
#[test]
fn spi_and_i2c_selection_use_the_same_single_device_rule_and_allow_multiple_controllers()
 {
    use crate::runtime::options::i2c::I2cOptions;
    use crate::runtime::options::spi::{SpiBus, SpiOptions};
    for count in [0, 2] {
        for spi in [false, true] {
            let mut options = HostOptions::default();
            if spi {
                options.spi.push(SpiOptions {
                    device: None,
                    host: Host::Linux(SpiBus {
                        path: "/dev/null".into(),
                        speed: 500_000,
                    }),
                    select: 0,
                    write_only: false,
                });
            } else {
                options.i2c.push(I2cOptions {
                    addresses: None,
                    device: None,
                    host: Host::Linux("/dev/zero".into()),
                });
            }
            let error =
                crate::runtime::host_bindings::prepare(&bus(count), options)
                    .err()
                    .unwrap()
                    .to_string();
            assert!(
                error.contains(if count == 0 {
                    "no "
                } else if spi {
                    "spi0, spi1"
                } else {
                    "i2c0, i2c1"
                }),
                "{error}"
            );
        }
    }
    for count in [1, 2] {
        let options = HostOptions {
            spi: vec![SpiOptions {
                device: (count == 2).then(|| "spi1".into()),
                host: Host::Linux(SpiBus {
                    path: "/dev/null".into(),
                    speed: 1_000_000,
                }),
                select: 2,
                write_only: true,
            }],
            i2c: vec![I2cOptions {
                addresses: None,
                device: (count == 2).then(|| "i2c1".into()),
                host: Host::Linux("/dev/zero".into()),
            }],
            ..HostOptions::default()
        };
        let connections =
            crate::runtime::host_bindings::prepare(&bus(count), options)
                .unwrap()
                .linux;
        assert!(connections[0].spi.as_ref().unwrap().write_only);
        assert_eq!(
            connections[0].spi.as_ref().unwrap().device,
            format!("spi{}", count - 1)
        );
        assert_eq!(
            connections[1].i2c.as_ref().unwrap().device,
            format!("i2c{}", count - 1)
        );
    }
}

fn tap(device: Option<String>, interface: &str) -> EthernetOptions {
    EthernetOptions {
        device,
        host: Host::Linux(interface.into()),
    }
}

#[test]
fn websocket_bindings_are_kept_apart_from_linux_ones_and_name_their_devices() {
    use crate::machine::host::websocket::{Address, Target};
    use crate::runtime::options::i2c::I2cOptions;
    use crate::runtime::options::spi::SpiOptions;

    let options = HostOptions {
        ethernet: vec![EthernetOptions::parse("ws").unwrap()],
        gpio: vec![GpioOptions::parse("ws,pins=5+6").unwrap()],
        i2c: vec![I2cOptions::parse("ws,addresses=0x48+0x68").unwrap()],
        spi: vec![
            SpiOptions::parse("ws,select=1,write-only=true").unwrap(),
            SpiOptions::parse("ws://h/bench,select=2").unwrap(),
        ],
        ..HostOptions::default()
    };
    let prepared =
        crate::runtime::host_bindings::prepare(&bus(1), options).unwrap();
    assert!(prepared.linux.is_empty());
    let found: Vec<(Vec<String>, Option<&str>)> = prepared
        .websocket
        .iter()
        .map(|binding| {
            let remote = match &binding.address {
                Address::Remote(url) => Some(url.as_str()),
                Address::Served => None,
            };
            (binding.names(), remote)
        })
        .collect();
    assert_eq!(
        found,
        [
            (vec!["gpio0".to_owned()], None),
            (vec!["ethernet0".to_owned()], None),
            (vec!["spi0/1".to_owned()], None),
            (vec!["spi0/2".to_owned()], Some("ws://h/bench")),
            (vec!["i2c0/0x48".to_owned(), "i2c0/0x68".to_owned()], None),
        ]
    );
    assert!(matches!(
        prepared.websocket[0].target,
        Target::Gpio { pins: 0x60 }
    ));
    assert!(matches!(
        prepared.websocket[2].target,
        Target::Spi {
            select: 1,
            write_only: true
        }
    ));
    assert!(matches!(
        &prepared.websocket[4].target,
        Target::I2c { addresses } if addresses == &[0x48, 0x68]
    ));
}
