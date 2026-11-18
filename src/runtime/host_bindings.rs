// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::uart::Uart;
use crate::machine::devices::{
    ethernet::Ethernet, gpio::Gpio, i2c::I2c, spi::Spi,
};
use crate::machine::host::device_selection::resolve;
use crate::machine::host::websocket::{
    Address, Served, Target, WebSocketBinding, WebSocketConnections,
};
use crate::machine::host::{HostConnections, I2cConnection, SpiConnection};

use crate::runtime::options::host::HostOptions;
use crate::runtime::options::socket::{Host, SocketOptions};
use crate::runtime::request::WebSettings;
use crate::runtime::uart::connections::UartConnections;
use crate::web::{Server, ServerSettings};
use std::io::Write;

pub(crate) type Hosts = (UartConnections, WebSocketConnections, Option<Server>);

#[derive(Default)]
pub(crate) struct Prepared {
    pub linux: Vec<HostConnections>,
    pub websocket: Vec<WebSocketBinding>,
}

fn binding(
    device: String,
    target: Target,
    socket: SocketOptions,
) -> WebSocketBinding {
    WebSocketBinding {
        address: socket.address,
        device,
        target,
        timeout: socket.timeout,
    }
}

pub(crate) fn connect(
    bus: &mut Bus,
    mut options: HostOptions,
    profile: &str,
    debugger: bool,
) -> Result<Hosts, EmulatorError> {
    let mut files = Vec::new();
    let mut links = Vec::new();
    let mut devices = HashSet::new();
    for option in std::mem::take(&mut options.uart) {
        match option.host {
            Host::Linux(host) => files.push((option.device, host)),
            Host::WebSocket(socket) => {
                let device =
                    resolve::<Uart>(bus, option.device.as_deref(), "UART")?;
                if !devices.insert(device.clone()) {
                    return Err(EmulatorError::Configuration(format!(
                        "duplicate UART binding for {device}"
                    )));
                }
                links.push(binding(device, Target::Uart, socket));
            }
        }
    }
    let web = options.web.clone();
    let mut prepared = prepare(bus, options)?;
    let mut uart = UartConnections::open(bus, files, &devices, debugger)?;
    for connection in prepared.linux {
        connection.connect(bus)?;
    }
    prepared.websocket.append(&mut links);
    let (server, served) = serve(&prepared.websocket, web)?;
    let mut websocket = WebSocketConnections::connect(
        bus,
        &prepared.websocket,
        profile,
        served,
    )?;
    uart.links = std::mem::take(&mut websocket.uart_inputs);
    uart.link_outputs = std::mem::take(&mut websocket.uart_outputs);
    Ok((uart, websocket, server))
}

pub(crate) fn prepare(
    bus: &Bus,
    options: HostOptions,
) -> Result<Prepared, EmulatorError> {
    let mut devices = HashSet::new();
    let mut interfaces = HashSet::new();
    let mut lines = HashSet::new();
    let mut prepared = Prepared::default();

    for option in options.gpio {
        let device = resolve::<Gpio>(bus, option.device.as_deref(), "GPIO")?;
        if !devices.insert(device.clone()) {
            return Err(EmulatorError::Configuration(format!(
                "duplicate GPIO binding for {device}; use one --gpio per \
                 device"
            )));
        }
        let chip = match option.host {
            Host::Linux(chip) => chip,
            Host::WebSocket(socket) => {
                let pins = option.pins.unwrap_or(u32::MAX);
                prepared.websocket.push(binding(
                    device,
                    Target::Gpio { pins },
                    socket,
                ));
                continue;
            }
        };
        for offset in &chip.lines {
            if !lines.insert((chip.path.clone(), *offset)) {
                return Err(EmulatorError::Configuration(
                    "GPIO line offsets must be unique across bindings on the \
                     same chip"
                        .into(),
                ));
            }
        }
        prepared.linux.push(HostConnections {
            gpio_chip: Some((device, chip.path)),
            gpio_lines: Some(chip.lines),
            gpio_settings: chip.settings,
            ..HostConnections::default()
        });
    }

    for option in options.ethernet {
        let device =
            resolve::<Ethernet>(bus, option.device.as_deref(), "Ethernet")?;
        if !devices.insert(device.clone()) {
            return Err(EmulatorError::Configuration(format!(
                "duplicate Ethernet binding for {device}; use one --ethernet \
                 per device"
            )));
        }
        let interface = match option.host {
            Host::Linux(interface) => interface,
            Host::WebSocket(socket) => {
                prepared.websocket.push(binding(
                    device,
                    Target::Ethernet,
                    socket,
                ));
                continue;
            }
        };
        if !interfaces.insert(interface.clone()) {
            return Err(EmulatorError::Configuration(
                "a host TAP interface may only be bound once".into(),
            ));
        }
        prepared.linux.push(HostConnections {
            ethernet_interface: Some((device, interface)),
            ..HostConnections::default()
        });
    }

    let mut paths = HashSet::new();
    let mut selections = HashSet::new();
    for option in options.spi {
        let device = resolve::<Spi>(bus, option.device.as_deref(), "SPI")?;
        let duplicate = || {
            EmulatorError::Configuration(
                "duplicate SPI device/select or host path".into(),
            )
        };
        if !selections.insert((device.clone(), option.select)) {
            return Err(duplicate());
        }
        let (select, write_only) = (option.select, option.write_only);
        let host = match option.host {
            Host::Linux(host) => host,
            Host::WebSocket(socket) => {
                prepared.websocket.push(binding(
                    device,
                    Target::Spi { select, write_only },
                    socket,
                ));
                continue;
            }
        };
        if !paths.insert(host.path.clone()) {
            return Err(duplicate());
        }
        prepared.linux.push(HostConnections {
            spi: Some(SpiConnection {
                device,
                path: host.path,
                select,
                speed: host.speed,
                write_only,
            }),
            ..HostConnections::default()
        });
    }
    for option in options.i2c {
        let device = resolve::<I2c>(bus, option.device.as_deref(), "I2C")?;
        let duplicate = || {
            EmulatorError::Configuration(
                "duplicate I2C device or host path".into(),
            )
        };
        if !devices.insert(device.clone()) {
            return Err(duplicate());
        }
        let path = match option.host {
            Host::Linux(path) => path,
            Host::WebSocket(socket) => {
                let addresses = option.addresses.unwrap_or_default();
                prepared.websocket.push(binding(
                    device,
                    Target::I2c { addresses },
                    socket,
                ));
                continue;
            }
        };
        if !paths.insert(path.clone()) {
            return Err(duplicate());
        }
        prepared.linux.push(HostConnections {
            i2c: Some(I2cConnection {
                addresses: option.addresses,
                device,
                path,
            }),
            ..HostConnections::default()
        });
    }

    for connection in &prepared.linux {
        connection.validate(bus)?;
    }
    Ok(prepared)
}

fn serve(
    bindings: &[WebSocketBinding],
    web: WebSettings,
) -> Result<(Option<Server>, Vec<Served>), EmulatorError> {
    let names: Vec<String> = bindings
        .iter()
        .filter(|binding| matches!(binding.address, Address::Served))
        .flat_map(WebSocketBinding::names)
        .collect();
    if names.is_empty() && web.root.is_none() {
        return Ok((None, Vec::new()));
    }
    if let Some(root) = web.root.as_deref().filter(|root| !root.is_dir()) {
        return Err(EmulatorError::Configuration(format!(
            "--web-root {} is not a folder",
            root.display()
        )));
    }
    let pages = web.root.is_some();
    let mut server = Server::start(ServerSettings {
        devices: names.clone(),
        host: web.host,
        port: web.port,
        root: web.root,
    })?;
    let mut notices = std::io::stderr().lock();
    if pages {
        writeln!(notices, "{:>12} {}", "Serving", server.url())?;
    }
    let mut served = Vec::new();
    for name in names {
        let url = server.device_url(&name);
        writeln!(notices, "{:>12} {url}", "Device")?;
        if let Some(stream) = server.take(&name) {
            served.push(Served { name, stream, url });
        }
    }
    Ok((Some(server), served))
}
