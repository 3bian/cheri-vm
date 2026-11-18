// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod slot;

use slot::Slot;

use super::privilege::pmp::Pmp;

use super::privilege::{Mode, Reach};

use crate::machine::bus::Bus;

use crate::machine::error::ExceptionCause;

const ACCESSED: u64 = 1 << 6;
pub(crate) const CAPABILITY_ENTRY: u64 = 1 << 58;
pub(crate) const CAPABILITY_FIELD: u64 = 0xf << 55;
const DIRTY: u64 = 1 << 7;
const EXECUTABLE: u64 = 1 << 3;
const EXECUTABLE_READABLE: u64 = 1 << 19;
const FRAME: u64 = (1 << 44) - 1;
const READABLE: u64 = 1 << 1;
const RESERVED: u64 = 0xffc0 << 48;
const SLOTS: usize = 256;
const SUPERVISOR_USER_MEMORY: u64 = 1 << 18;
const USER: u64 = 1 << 4;
const VALID: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 2;

pub(crate) struct Translation {
    capability_field: u64,
    pub(crate) last: Option<(u8, bool)>,
    root: u64,
    slots: Box<[Slot; SLOTS]>,
}

impl Translation {
    pub(crate) fn enabled(&self) -> bool {
        self.root >> 60 != 0
    }

    pub(crate) fn flush(&mut self) {
        self.slots.fill(Slot::default());
    }

    pub(crate) fn inspect(&self, bus: &Bus, address: u64) -> Option<u64> {
        if ((address as i64) << 25) >> 25 != address as i64 {
            return None;
        }
        let mut table = (self.root & FRAME) << 12;
        for level in (0..3).rev() {
            let entry_address =
                table + ((address >> (12 + 9 * level)) & 0x1ff) * 8;
            let half = |offset| {
                bus.inspect(entry_address + offset, 4).ok().map(u64::from)
            };
            let entry = half(0)? | half(4)? << 32;
            if entry & VALID == 0 {
                return None;
            }
            let frame = (entry >> 10) & FRAME;
            if entry & (READABLE | EXECUTABLE) == 0 {
                table = frame << 12;
                continue;
            }
            let span = (1_u64 << (9 * level)) - 1;
            return Some(
                ((frame | ((address >> 12) & span)) << 12) | (address & 0xfff),
            );
        }
        None
    }

    pub(crate) fn resolve(
        &mut self,
        bus: &mut Bus,
        pmp: &Pmp,
        address: u64,
        reach: Reach,
        mode: Mode,
        status: u64,
    ) -> Result<u64, ExceptionCause> {
        if ((address as i64) << 25) >> 25 != address as i64 {
            return Err(reach.page_fault());
        }
        let page = address >> 12;
        let index = page as usize % SLOTS;
        let held = self.slots[index];
        if held.tag == page + 1 && permitted(held.flags, reach, mode, status) {
            self.last = Some(((held.flags >> 8) as u8, held.flags & USER != 0));
            return Ok(held.frame | (address & 0xfff));
        }
        let (flags, frame) = self.walk(bus, pmp, address, reach)?;
        if !permitted(flags, reach, mode, status) {
            self.slots[index] = Slot::default();
            return Err(reach.page_fault());
        }
        self.last = Some(((flags >> 8) as u8, flags & USER != 0));
        self.slots[index] = Slot {
            flags,
            frame,
            tag: page + 1,
        };
        Ok(frame | (address & 0xfff))
    }

    pub(crate) fn root(&self) -> u64 {
        self.root
    }

    pub(crate) fn set_root(&mut self, value: u64) {
        if !matches!(value >> 60, 0 | 8) {
            return;
        }
        self.root = value & ((0xf << 60) | (0xffff << 44) | FRAME);
        self.flush();
    }

    fn walk(
        &self,
        bus: &mut Bus,
        pmp: &Pmp,
        address: u64,
        reach: Reach,
    ) -> Result<(u64, u64), ExceptionCause> {
        let reserved = RESERVED & !self.capability_field;
        let mut table = (self.root & FRAME) << 12;
        for level in (0..3).rev() {
            let entry_address =
                table + ((address >> (12 + 9 * level)) & 0x1ff) * 8;
            if !pmp.allows(entry_address, 8, Reach::Load, Mode::Supervisor) {
                return Err(reach.access_fault());
            }
            let half = |bus: &mut Bus, offset| {
                bus.read(entry_address + offset, 4)
                    .map(u64::from)
                    .map_err(|_| reach.access_fault())
            };
            let entry = half(bus, 0)? | half(bus, 4)? << 32;
            let leaf = entry & (READABLE | EXECUTABLE) != 0;
            if entry & VALID == 0
                || entry & (READABLE | WRITABLE) == WRITABLE
                || entry & reserved != 0
                || (!leaf && entry & (ACCESSED | DIRTY | USER) != 0)
            {
                return Err(reach.page_fault());
            }
            let frame = (entry >> 10) & FRAME;
            if !leaf {
                table = frame << 12;
                continue;
            }
            let span = (1_u64 << (9 * level)) - 1;
            if frame & span != 0 || entry & ACCESSED == 0 {
                return Err(reach.page_fault());
            }
            let flags = (entry & 0xff) | (entry & CAPABILITY_FIELD) >> 47;
            return Ok((flags, (frame | ((address >> 12) & span)) << 12));
        }
        Err(reach.page_fault())
    }

    pub(crate) fn with_capability_field(capability_field: u64) -> Self {
        Self {
            capability_field,
            ..Self::default()
        }
    }
}

impl Default for Translation {
    fn default() -> Self {
        Self {
            capability_field: 0,
            last: None,
            root: 0,
            slots: Box::new([Slot::default(); SLOTS]),
        }
    }
}

fn permitted(flags: u64, reach: Reach, mode: Mode, status: u64) -> bool {
    let user = flags & USER != 0;
    let data = if mode == Mode::User {
        user
    } else {
        !user || status & SUPERVISOR_USER_MEMORY != 0
    };
    match reach {
        Reach::Fetch => flags & EXECUTABLE != 0 && user == (mode == Mode::User),
        Reach::Load => {
            data && (flags & READABLE != 0
                || (status & EXECUTABLE_READABLE != 0
                    && flags & EXECUTABLE != 0))
        }
        Reach::Store => data && flags & WRITABLE != 0 && flags & DIRTY != 0,
    }
}
