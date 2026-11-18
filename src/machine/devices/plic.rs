// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{
    Device, DeviceActivity, InterruptLine, register_width,
};
use crate::machine::error::EmulatorError;

pub(crate) const BYTES: u32 = 0x400_0000;
const CLAIM: u32 = 4;
const CONTEXT: u32 = 0x20_0000;
const CONTEXT_STRIDE: u32 = 0x1000;
const CONTEXTS: usize = 2;
const ENABLE: u32 = 0x2000;
const ENABLE_STRIDE: u32 = 0x80;
const HIGHEST_PRIORITY: u32 = 7;
const PENDING: u32 = 0x1000;
pub(crate) const SOURCES: u32 = 30;

#[derive(Default)]
pub(crate) struct Plic {
    claimed: u32,
    enabled: [u32; CONTEXTS],
    levels: u32,
    priorities: [u8; 32],
    thresholds: [u8; CONTEXTS],
}

impl Plic {
    fn best(&self, context: usize) -> Option<u32> {
        let waiting = self.pending() & self.enabled[context];
        (1..=SOURCES)
            .filter(|source| waiting & (1 << source) != 0)
            .filter(|&source| {
                self.priorities[source as usize] > self.thresholds[context]
            })
            .max_by_key(|&source| {
                (self.priorities[source as usize], u32::MAX - source)
            })
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn pending(&self) -> u32 {
        self.levels & !self.claimed
    }

    pub(crate) fn set_levels(&mut self, levels: u32) {
        self.levels = levels & Self::sources();
    }

    fn sources() -> u32 {
        ((1_u64 << (SOURCES + 1)) - 2) as u32
    }
}

impl Device for Plic {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: &[
                InterruptLine::MachineExternal,
                InterruptLine::SupervisorExternal,
            ],
            ..DeviceActivity::default()
        }
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        if self.best(0).is_some() {
            Some(InterruptLine::MachineExternal)
        } else {
            self.best(1).map(|_| InterruptLine::SupervisorExternal)
        }
    }

    fn interrupt_pending(&self, line: InterruptLine) -> bool {
        match line {
            InterruptLine::MachineExternal => self.best(0).is_some(),
            InterruptLine::SupervisorExternal => self.best(1).is_some(),
            InterruptLine::MachineSoftware | InterruptLine::MachineTimer => {
                false
            }
        }
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        register_width(offset, width, BYTES)?;
        if let Some((context, register)) = context_of(offset) {
            return Ok(match register {
                0 => u32::from(self.thresholds[context]),
                CLAIM => {
                    let source = self.best(context).unwrap_or(0);
                    self.claimed |= (1 << source) & Self::sources();
                    source
                }
                _ => 0,
            });
        }
        if let Some(context) = enable_of(offset) {
            return Ok(self.enabled[context]);
        }
        Ok(match offset {
            PENDING => self.pending(),
            offset if offset <= SOURCES * 4 => {
                u32::from(self.priorities[(offset / 4) as usize])
            }
            _ => 0,
        })
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        register_width(offset, width, BYTES)?;
        if let Some((context, register)) = context_of(offset) {
            match register {
                0 => {
                    self.thresholds[context] = (value & HIGHEST_PRIORITY) as u8
                }
                CLAIM if (1..=SOURCES).contains(&value) => {
                    self.claimed &= !(1 << value)
                }
                _ => {}
            }
        } else if let Some(context) = enable_of(offset) {
            self.enabled[context] = value & Self::sources();
        } else if (4..=SOURCES * 4).contains(&offset) {
            self.priorities[(offset / 4) as usize] =
                (value & HIGHEST_PRIORITY) as u8;
        }
        Ok(())
    }
}

fn context_of(offset: u32) -> Option<(usize, u32)> {
    let context = (offset.checked_sub(CONTEXT)? / CONTEXT_STRIDE) as usize;
    (context < CONTEXTS).then_some((context, offset % CONTEXT_STRIDE))
}

fn enable_of(offset: u32) -> Option<usize> {
    let context = (offset.checked_sub(ENABLE)? / ENABLE_STRIDE) as usize;
    (context < CONTEXTS && offset.is_multiple_of(ENABLE_STRIDE))
        .then_some(context)
}
