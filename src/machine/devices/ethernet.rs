// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::{EmulatorError, HostErrorKind};

mod frame_transport;
mod network_backend;
pub(crate) mod registers;

pub(crate) use frame_transport::FrameTransport;
pub(crate) use network_backend::NetworkBackend;
use registers::{
    RECEIVE_POP, Register, STATUS_ALL, STATUS_RECEIVE_READY,
    STATUS_TRANSFER_ERROR, STATUS_TRANSMIT_COMPLETE, TRANSMIT_SEND,
};

pub(crate) const MAX_FRAME: usize = 1518;
const MIN_HOST_FRAME: usize = 14;
pub(crate) const QUEUE_CAPACITY: usize = 8;

pub(crate) struct Ethernet {
    pub(crate) backend: NetworkBackend,
    host_error: Option<String>,
    interrupt_mask: u32,
    pub(crate) link: bool,
    outgoing_buffer: Vec<u8>,
    read_cursor: usize,
    pub(crate) received: VecDeque<Vec<u8>>,
    pub(crate) status: u32,
    transmit_length: usize,
    pub(crate) transmitted: VecDeque<Vec<u8>>,
}

impl Ethernet {
    pub(crate) fn new(backend: NetworkBackend) -> Self {
        Self {
            link: true,
            backend,
            host_error: None,
            interrupt_mask: 0,
            transmit_length: 0,
            outgoing_buffer: Vec::new(),
            transmitted: VecDeque::new(),
            received: VecDeque::new(),
            read_cursor: 0,
            status: 0,
        }
    }

    fn poll_host(&mut self) {
        if !self.link {
            return;
        }

        let NetworkBackend::Host(transport) = &mut self.backend else {
            return;
        };
        let mut buffer = [0_u8; MAX_FRAME + 1];

        for _ in 0..QUEUE_CAPACITY {
            match transport.receive(&mut buffer) {
                Ok(length)
                    if (MIN_HOST_FRAME..=MAX_FRAME).contains(&length)
                        && self.received.len() < QUEUE_CAPACITY =>
                {
                    self.received.push_back(buffer[..length].to_vec());
                    self.status |= STATUS_RECEIVE_READY;
                }
                Ok(_) => self.status |= STATUS_TRANSFER_ERROR,
                Err(error) if error.kind() == HostErrorKind::WouldBlock => {
                    break;
                }
                Err(error) if error.kind() == HostErrorKind::Interrupted => {
                    continue;
                }
                Err(error) if error.kind() == HostErrorKind::NotConnected => {
                    self.host_error = Some(error.to_string());
                    break;
                }
                Err(_) => {
                    self.status |= STATUS_TRANSFER_ERROR;
                    break;
                }
            }
        }
    }

    fn transmit_frame(&mut self) -> bool {
        match &mut self.backend {
            NetworkBackend::Host(transport) => {
                self.outgoing_buffer.len() >= MIN_HOST_FRAME
                    && transport.transmit(&self.outgoing_buffer).is_ok_and(
                        |written| written == self.outgoing_buffer.len(),
                    )
            }
            NetworkBackend::InMemory => {
                if self.transmitted.len() >= QUEUE_CAPACITY {
                    return false;
                }

                self.transmitted.push_back(self.outgoing_buffer.clone());
                true
            }
        }
    }
}

impl Device for Ethernet {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: matches!(self.backend, NetworkBackend::Host(_)),
            interrupt_lines: &[InterruptLine::MachineExternal],
            ..DeviceActivity::default()
        }
    }

    fn host_error(&self) -> Option<&str> {
        self.host_error.as_deref()
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        (self.status & self.interrupt_mask != 0)
            .then_some(InterruptLine::MachineExternal)
    }

    fn poll_host_inputs(&mut self) {
        self.poll_host();
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::Link => u32::from(self.link),
            Register::Status => self.status,
            Register::TransmitLength => self.transmit_length as u32,
            Register::TransmitData => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "TX_DATA is write-only".into(),
                });
            }
            Register::TransmitSend | Register::ReceivePop => 0,
            Register::ReceiveLength => {
                self.received.front().map_or(0, |frame| frame.len() as u32)
            }
            Register::ReceiveData => {
                if let Some(frame) = self.received.front() {
                    let mut value = 0_u32;

                    for index in 0..4 {
                        if let Some(byte) = frame.get(self.read_cursor + index)
                        {
                            value |= (*byte as u32) << (index * 8);
                        }
                    }

                    self.read_cursor = (self.read_cursor + 4).min(frame.len());
                    value
                } else {
                    0
                }
            }
            Register::InterruptMask => self.interrupt_mask,
        })
    }

    fn tick(&mut self, _instructions: u64) {
        self.poll_host();
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match Register::access(offset, width)? {
            Register::Status => {
                self.status &= !(value
                    & (STATUS_TRANSMIT_COMPLETE | STATUS_TRANSFER_ERROR));
            }
            Register::TransmitLength => {
                if value == 0 || value as usize > MAX_FRAME {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "Ethernet frame length must be 1..1518"
                            .into(),
                    });
                }

                self.transmit_length = value as usize;
                self.outgoing_buffer.clear();
            }
            Register::TransmitData => {
                if self.transmit_length == 0 {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "configure frame length before writing \
                                      TX_DATA"
                            .into(),
                    });
                }

                for byte in value.to_le_bytes() {
                    if self.outgoing_buffer.len() < self.transmit_length {
                        self.outgoing_buffer.push(byte);
                    }
                }
            }
            Register::TransmitSend => {
                if value != TRANSMIT_SEND {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "TX_SEND accepts 1".into(),
                    });
                }

                if self.transmit_length != 0
                    && self.outgoing_buffer.len() == self.transmit_length
                    && self.link
                    && self.transmit_frame()
                {
                    self.status |= STATUS_TRANSMIT_COMPLETE;
                } else {
                    self.status |= STATUS_TRANSFER_ERROR;
                }

                self.outgoing_buffer.clear();
                self.transmit_length = 0;
            }
            Register::ReceivePop => {
                if value & RECEIVE_POP != 0 {
                    self.received.pop_front();
                    self.read_cursor = 0;

                    if self.received.is_empty() {
                        self.status &= !STATUS_RECEIVE_READY;
                    }
                }
            }
            Register::InterruptMask => self.interrupt_mask = value & STATUS_ALL,
            Register::Link
            | Register::ReceiveLength
            | Register::ReceiveData => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "Ethernet register is read-only".into(),
                });
            }
        }

        Ok(())
    }
}
