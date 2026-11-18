// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::connection::Connection;
use super::super::ethernet::WebSocketFrames;
use super::super::gpio::WebSocketLines;
use super::super::i2c::WebSocketI2c;
use super::super::link::Link;
use super::super::spi::WebSocketSpi;
use super::super::uart::{WebSocketUartInput, WebSocketUartOutput};
use super::{Address, Served, Target, WebSocketBinding};
use crate::hart::execution::{ExecutionReport, StopReason};
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::ethernet::{Ethernet, NetworkBackend};
use crate::machine::devices::gpio::Gpio;
use crate::machine::devices::i2c::I2c;
use crate::machine::devices::spi::Spi;
use crate::websocket::message::Reason;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct WebSocketConnections {
    pub(super) connections: Vec<Rc<RefCell<Connection>>>,
    pub(crate) uart_inputs: Vec<WebSocketUartInput>,
    pub(crate) uart_outputs: Vec<WebSocketUartOutput>,
}

impl WebSocketConnections {
    pub(crate) fn connect(
        bus: &mut Bus,
        bindings: &[WebSocketBinding],
        profile: &str,
        mut served: Vec<Served>,
    ) -> Result<Self, EmulatorError> {
        let mut connections = Self::default();
        let waited = Rc::new(Cell::new(Duration::ZERO));
        let last = Rc::new(RefCell::new(None));
        let mut opened = Vec::new();
        for binding in bindings {
            let (kind, write_only) = binding.kind();
            let mut links = Vec::new();
            for name in binding.names() {
                let mut connection = match &binding.address {
                    Address::Remote(url) => {
                        Connection::open(url, &name, waited.clone())?
                    }
                    Address::Served => {
                        let index = served
                            .iter()
                            .position(|served| served.name == name)
                            .ok_or_else(|| {
                                EmulatorError::Configuration(format!(
                                    "{name} is not served"
                                ))
                            })?;
                        let served = served.remove(index);
                        Connection::served(
                            served.stream,
                            &served.url,
                            &name,
                            waited.clone(),
                        )?
                    }
                };
                connection.announce(profile, kind, write_only)?;
                let connection = Rc::new(RefCell::new(connection));
                connections.connections.push(connection.clone());
                links.push(Link {
                    connection,
                    last: last.clone(),
                    timeout: binding.timeout,
                });
            }
            opened.push((binding, links));
        }
        for connection in &connections.connections {
            connection.borrow_mut().await_ready()?;
        }
        if !bindings.is_empty() {
            bus.add_host_wait(waited);
        }
        for (binding, links) in opened {
            connections.install(bus, binding, links)?;
        }
        Ok(connections)
    }

    pub(crate) fn finish(
        &self,
        outcome: Result<&ExecutionReport, &EmulatorError>,
    ) {
        let (reason, message) = match outcome {
            Ok(report) => match report.stop {
                StopReason::GuestCompletion { code } => {
                    (Reason::Finished, format!("exit code {code}"))
                }
                StopReason::BudgetExhausted => {
                    (Reason::Stopped, "step limit reached".to_owned())
                }
                StopReason::Breakpoint { address }
                | StopReason::GuestEbreak { address } => {
                    (Reason::Stopped, format!("breakpoint at 0x{address:08x}"))
                }
            },
            Err(error) => (Reason::Failed, error.to_string()),
        };
        for connection in &self.connections {
            connection.borrow_mut().finish(reason, &message);
        }
    }

    pub(crate) fn handle(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            ..Self::default()
        }
    }

    pub(super) fn install(
        &mut self,
        bus: &mut Bus,
        binding: &WebSocketBinding,
        mut links: Vec<Link>,
    ) -> Result<(), EmulatorError> {
        let missing = |kind: &str| {
            EmulatorError::Configuration(format!(
                "no {kind} device named {}",
                binding.device
            ))
        };
        let Some(link) = links.first().cloned() else {
            return Ok(());
        };
        match &binding.target {
            Target::Ethernet => {
                let transport = WebSocketFrames { link };
                *bus.device_mut::<Ethernet>(&binding.device)
                    .ok_or_else(|| missing("Ethernet"))? =
                    Ethernet::new(NetworkBackend::Host(Box::new(transport)));
            }
            Target::Gpio { pins } => {
                let transport = WebSocketLines::new(link, *pins);
                *bus.device_mut::<Gpio>(&binding.device)
                    .ok_or_else(|| missing("GPIO"))? =
                    Gpio::with_host(Box::new(transport))?;
            }
            Target::I2c { addresses } => {
                let transport = WebSocketI2c {
                    device: binding.device.clone(),
                    links: addresses
                        .iter()
                        .copied()
                        .zip(links.drain(..))
                        .collect(),
                };
                bus.device_mut::<I2c>(&binding.device)
                    .ok_or_else(|| missing("I2C"))?
                    .share_with_host(Box::new(transport), addresses.clone())?;
            }
            Target::Spi { select, write_only } => {
                let transport = WebSocketSpi {
                    link,
                    write_only: *write_only,
                };
                bus.device_mut::<Spi>(&binding.device)
                    .ok_or_else(|| missing("SPI"))?
                    .replace_peripheral(*select, Box::new(transport))?;
            }
            Target::Uart => {
                self.uart_inputs.push(WebSocketUartInput {
                    device: binding.device.clone(),
                    link: link.clone(),
                    pending: VecDeque::new(),
                });
                self.uart_outputs.push(WebSocketUartOutput {
                    device: binding.device.clone(),
                    link,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn settle(&self) -> Result<(), EmulatorError> {
        for connection in &self.connections {
            connection.borrow_mut().settle()?;
        }
        Ok(())
    }
}
