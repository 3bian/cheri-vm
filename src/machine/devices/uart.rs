// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;

use super::output_capture::OutputCapture;
use crate::machine::devices::{
    Device, DeviceActivity, InterruptLine, register_width,
};
use crate::machine::error::EmulatorError;

mod layout;
pub(crate) mod registers;

pub(crate) use layout::UartLayout;
use registers::{
    CONTROL_INTERRUPT_ENABLE, ENABLE_RECEIVED, ENABLE_TRANSMITTED,
    IDENTITY_NONE, IDENTITY_RECEIVED, IDENTITY_TRANSMITTED,
    LINE_CONTROL_DIVISOR_LATCH, LINE_STATUS_DATA_READY,
    LINE_STATUS_TRANSMIT_EMPTY, NS16550_BYTES, Ns16550Register, Register,
    STATUS_RECEIVE_READY, STATUS_TRANSMIT_READY,
};

#[derive(Default)]
pub(crate) struct Uart {
    control: u32,
    divisor: [u8; 2],
    divisor_latch: bool,
    incoming: VecDeque<u8>,
    layout: UartLayout,
    line_control: u32,
    scratch: u32,
    transmitted_pending: bool,
    pub(crate) transmitted: OutputCapture,
}

impl Uart {
    pub(crate) const RECEIVE_CAPACITY: usize = 4096;

    pub(crate) fn drain_transmitted(&mut self, output: &mut Vec<u8>) {
        self.transmitted.drain_into(output);
    }

    fn identity(&self) -> u32 {
        if self.control & ENABLE_RECEIVED != 0 && !self.incoming.is_empty() {
            IDENTITY_RECEIVED
        } else if self.control & ENABLE_TRANSMITTED != 0
            && self.transmitted_pending
        {
            IDENTITY_TRANSMITTED
        } else {
            IDENTITY_NONE
        }
    }

    pub(crate) fn inject_receive(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), EmulatorError> {
        if bytes.len()
            > Self::RECEIVE_CAPACITY.saturating_sub(self.incoming.len())
        {
            return Err(EmulatorError::Configuration(
                "UART receive FIFO exceeds 4096 bytes".into(),
            ));
        }

        self.incoming.extend(bytes);

        Ok(())
    }

    pub(crate) fn receive_remaining(&self) -> usize {
        Self::RECEIVE_CAPACITY - self.incoming.len()
    }

    fn transmit(&mut self, value: u32) -> Result<(), EmulatorError> {
        if !self.transmitted.push(value as u8) {
            return Err(EmulatorError::Bus {
                address: 0,
                explanation: "UART capture buffer full; host must drain \
                              transmitted bytes"
                    .into(),
            });
        }
        Ok(())
    }

    pub(crate) fn with_layout(layout: UartLayout) -> Self {
        Self {
            layout,
            ..Self::default()
        }
    }
}

impl Device for Uart {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: match self.layout {
                UartLayout::Ns16550 => &[],
                UartLayout::Simple => &[InterruptLine::MachineExternal],
            },
            ..DeviceActivity::default()
        }
    }

    fn drain_output(&mut self, output: &mut Vec<u8>) -> Option<&'static str> {
        self.drain_transmitted(output);
        Some("UART")
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        let raised = match self.layout {
            UartLayout::Ns16550 => self.identity() != IDENTITY_NONE,
            UartLayout::Simple => {
                self.control & CONTROL_INTERRUPT_ENABLE != 0
                    && !self.incoming.is_empty()
            }
        };
        raised.then_some(InterruptLine::MachineExternal)
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        if self.layout == UartLayout::Ns16550 {
            return Ok(match Ns16550Register::access(offset, width)? {
                Ns16550Register::Data if self.divisor_latch => {
                    u32::from(self.divisor[0])
                }
                Ns16550Register::Data => {
                    u32::from(self.incoming.pop_front().unwrap_or(0))
                }
                Ns16550Register::InterruptEnable if self.divisor_latch => {
                    u32::from(self.divisor[1])
                }
                Ns16550Register::InterruptEnable => self.control,
                Ns16550Register::InterruptIdentity => {
                    let identity = self.identity();
                    self.transmitted_pending &=
                        identity != IDENTITY_TRANSMITTED;
                    identity
                }
                Ns16550Register::LineControl => self.line_control,
                Ns16550Register::LineStatus => {
                    bit_if(
                        self.transmitted.has_capacity(),
                        LINE_STATUS_TRANSMIT_EMPTY,
                    ) | bit_if(
                        !self.incoming.is_empty(),
                        LINE_STATUS_DATA_READY,
                    )
                }
                Ns16550Register::Scratch => self.scratch,
                Ns16550Register::ModemControl
                | Ns16550Register::ModemStatus => 0,
            });
        }
        Ok(match Register::access(offset, width)? {
            Register::Data => u32::from(self.incoming.pop_front().unwrap_or(0)),
            Register::Status => {
                bit_if(!self.incoming.is_empty(), STATUS_RECEIVE_READY)
                    | bit_if(
                        self.transmitted.has_capacity(),
                        STATUS_TRANSMIT_READY,
                    )
            }
            Register::Control => self.control,
        })
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        if self.layout == UartLayout::Ns16550 {
            register_width(offset, width, NS16550_BYTES)?;
            match Ns16550Register::find(offset) {
                Some(Ns16550Register::Data) if self.divisor_latch => {
                    self.divisor[0] = value as u8;
                }
                Some(Ns16550Register::Data) => {
                    self.transmit(value)?;
                    self.transmitted_pending = true;
                }
                Some(Ns16550Register::InterruptEnable)
                    if self.divisor_latch =>
                {
                    self.divisor[1] = value as u8;
                }
                Some(Ns16550Register::InterruptEnable) => {
                    let enabled =
                        value & (ENABLE_RECEIVED | ENABLE_TRANSMITTED);
                    self.transmitted_pending |=
                        enabled & !self.control & ENABLE_TRANSMITTED != 0;
                    self.control = enabled;
                }
                Some(Ns16550Register::LineControl) => {
                    self.line_control = value & 0xff;
                    self.divisor_latch =
                        value & LINE_CONTROL_DIVISOR_LATCH != 0;
                }
                Some(Ns16550Register::Scratch) => self.scratch = value & 0xff,
                _ => (),
            }
            return Ok(());
        }
        match Register::access(offset, width)? {
            Register::Data => self.transmit(value)?,
            Register::Control => {
                self.control = value & CONTROL_INTERRUPT_ENABLE
            }
            Register::Status => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "UART register is read-only".into(),
                });
            }
        }

        Ok(())
    }
}

fn bit_if(condition: bool, bit: u32) -> u32 {
    if condition { bit } else { 0 }
}
