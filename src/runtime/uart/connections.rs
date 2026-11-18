// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashSet;
use std::path::Path;

use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::uart::Uart;
use crate::machine::host::device_selection;
use crate::machine::host::uart::input::UartInput;
use crate::machine::host::uart::port::UartPort;

use crate::machine::host::websocket::uart::{
    WebSocketUartInput, WebSocketUartOutput,
};
use crate::runtime::options::uart::UartFiles;
use crate::runtime::uart::output_binding::UartOutputBinding;
use crate::runtime::uart::{identity, is_stdin_path, is_stdout_path};

#[derive(Default)]
pub(crate) struct UartConnections {
    pub inputs: Vec<UartInput>,
    pub link_outputs: Vec<WebSocketUartOutput>,
    pub links: Vec<WebSocketUartInput>,
    pub outputs: Vec<UartOutputBinding>,
    ports: Vec<UartPort>,
}

impl UartConnections {
    pub(crate) fn open(
        bus: &Bus,
        options: Vec<(Option<String>, UartFiles)>,
        taken: &HashSet<String>,
        debugger: bool,
    ) -> Result<Self, EmulatorError> {
        let mut devices = taken.clone();
        let mut inputs = HashSet::new();
        let mut outputs = HashSet::new();
        let mut bindings = Vec::new();
        for (device, option) in options {
            let device = device_selection::resolve::<Uart>(
                bus,
                device.as_deref(),
                "UART",
            )?;
            if !devices.insert(device.clone()) {
                return Err(EmulatorError::Configuration(format!(
                    "duplicate UART binding for {device}"
                )));
            }
            if option
                .input
                .as_ref()
                .is_some_and(|path| !inputs.insert(path.clone()))
            {
                return Err(EmulatorError::Configuration(
                    "a host UART input path may only be bound once".into(),
                ));
            }
            if option.output.as_ref().is_some_and(|path| {
                !is_stdout_path(path) && !outputs.insert(path.clone())
            }) {
                return Err(EmulatorError::Configuration(
                    "a host UART output path may only be bound once; use \
                     stdout for combined output"
                        .into(),
                ));
            }
            bindings.push((device, option));
        }
        let mut connections = Self::default();
        let mut configured_ports: HashSet<u64> = HashSet::new();
        for (_, option) in &bindings {
            let Some(settings) = &option.settings else {
                continue;
            };
            let mut ports: HashSet<u64> = HashSet::new();
            for path in option.input.iter().chain(option.output.iter()) {
                if is_stdout_path(path) || is_stdin_path(path) {
                    continue;
                }
                if let Some(port) = identity::character_device(Path::new(path))
                    && ports.insert(port)
                {
                    if !configured_ports.insert(port) {
                        return Err(EmulatorError::Configuration(
                            "a host UART port may only be configured by one \
                             binding"
                                .into(),
                        ));
                    }
                    connections
                        .ports
                        .push(UartPort::open(Path::new(path), settings)?);
                }
            }
            if ports.is_empty() {
                return Err(EmulatorError::Configuration(
                    "UART baud/framing/flow settings require a host TTY path; \
                     stdin/stdout and files keep their existing settings"
                        .into(),
                ));
            }
        }
        let mut output_files: HashSet<(u64, u64)> = HashSet::new();
        output_files.extend(identity::standard_output_file());
        for (device, option) in bindings {
            if let Some(path) = option.input {
                connections.inputs.push(UartInput::open(
                    bus,
                    &device,
                    Path::new(&path),
                )?);
            }
            if let Some(path) = option.output
                && let Some(output) =
                    UartOutputBinding::open(device, &path, debugger)?
            {
                if let Some(file) = identity::of(&output.file)?
                    && !output_files.insert(file)
                {
                    return Err(EmulatorError::Configuration(
                        "a host UART output destination may only be bound \
                         once; use stdout for combined output"
                            .into(),
                    ));
                }
                connections.outputs.push(output);
            }
        }
        if !debugger {
            for output in &connections.outputs {
                output.truncate()?;
            }
        }
        Ok(connections)
    }
    pub(crate) fn restore(&mut self) -> Result<(), EmulatorError> {
        let mut result = Ok(());
        for port in &mut self.ports {
            let restored = port.restore();
            if result.is_ok() {
                result = restored;
            }
        }
        result
    }
}
