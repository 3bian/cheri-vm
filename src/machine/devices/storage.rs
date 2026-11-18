// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

pub(crate) mod registers;

use registers::{
    COMMAND_READ, COMMAND_WRITE, CONTROL_INTERRUPT_ENABLE, Register,
    SECTOR_BYTES, SECTOR_WORDS, STATUS_COMPLETE, STATUS_ERROR, STATUS_PRESENT,
};

pub(crate) struct Storage {
    buffer: [u8; SECTOR_BYTES],
    control: u32,
    pub(crate) media: Option<Vec<u8>>,
    read_only: bool,
    sector: u32,
    pub(crate) status: u32,
    word_index: usize,
}

impl Storage {
    pub(crate) fn new(
        sectors: usize,
        read_only: bool,
    ) -> Result<Self, EmulatorError> {
        if sectors == 0 || sectors > 65_536 {
            return Err(EmulatorError::Configuration(
                "storage size must be 1..65536 sectors".into(),
            ));
        }

        Ok(Self {
            media: Some(vec![0; sectors * SECTOR_BYTES]),
            read_only,
            sector: 0,
            word_index: 0,
            buffer: [0; SECTOR_BYTES],
            status: STATUS_PRESENT,
            control: 0,
        })
    }
}

impl Device for Storage {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: &[InterruptLine::MachineExternal],
            ..DeviceActivity::default()
        }
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        (self.control & CONTROL_INTERRUPT_ENABLE != 0
            && self.status & (STATUS_COMPLETE | STATUS_ERROR) != 0)
            .then_some(InterruptLine::MachineExternal)
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::Sector => self.sector,
            Register::WordIndex => self.word_index as u32,
            Register::Data => u32::from_le_bytes(
                self.buffer[self.word_index * 4..self.word_index * 4 + 4]
                    .try_into()
                    .map_err(|_| {
                        EmulatorError::Configuration(
                            "invalid storage word index".into(),
                        )
                    })?,
            ),
            Register::Command => 0,
            Register::Status => self.status,
            Register::Control => self.control,
        })
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match Register::access(offset, width)? {
            Register::Sector => self.sector = value,
            Register::WordIndex => {
                if value >= SECTOR_WORDS {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "storage word index must be 0..127".into(),
                    });
                }

                self.word_index = value as usize;
            }
            Register::Data => self.buffer
                [self.word_index * 4..self.word_index * 4 + 4]
                .copy_from_slice(&value.to_le_bytes()),
            Register::Command => {
                self.status = 0;
                let Some(media) = &mut self.media else {
                    self.status = STATUS_ERROR;
                    return Ok(());
                };
                let start = self.sector as u64 * SECTOR_BYTES as u64;

                if start + SECTOR_BYTES as u64 > media.len() as u64 {
                    self.status = STATUS_PRESENT | STATUS_ERROR;
                } else if value == COMMAND_READ {
                    self.buffer.copy_from_slice(
                        &media[start as usize..start as usize + SECTOR_BYTES],
                    );
                    self.status = STATUS_PRESENT | STATUS_COMPLETE;
                } else if value == COMMAND_WRITE && !self.read_only {
                    media[start as usize..start as usize + SECTOR_BYTES]
                        .copy_from_slice(&self.buffer);
                    self.status = STATUS_PRESENT | STATUS_COMPLETE;
                } else {
                    self.status = STATUS_PRESENT | STATUS_ERROR;
                }
            }
            Register::Status => {
                self.status &= !(value & (STATUS_COMPLETE | STATUS_ERROR))
            }
            Register::Control => {
                self.control = value & CONTROL_INTERRUPT_ENABLE
            }
        }

        Ok(())
    }
}
