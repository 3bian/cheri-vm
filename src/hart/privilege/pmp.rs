// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Mode, Reach};

const ADDRESS_BITS: u64 = (1 << 54) - 1;
pub(crate) const ENTRIES: usize = 8;
const LOCKED: u8 = 1 << 7;
const MATCHING: u8 = 3 << 3;
const TOP_OF_RANGE: u8 = 1 << 3;

#[derive(Default)]
pub(crate) struct Pmp {
    addresses: [u64; ENTRIES],
    configurations: [u8; ENTRIES],
    ranges: [Option<(u64, u64)>; ENTRIES],
}

impl Pmp {
    pub(crate) fn address(&self, index: usize) -> u64 {
        self.addresses.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn allows(
        &self,
        address: u64,
        width: usize,
        reach: Reach,
        mode: Mode,
    ) -> bool {
        let last = address.wrapping_add((width as u64).saturating_sub(1));
        for (range, configuration) in
            self.ranges.iter().zip(self.configurations)
        {
            let Some((low, high)) = *range else {
                continue;
            };
            let first_inside = (low..=high).contains(&address);
            let last_inside = (low..=high).contains(&last);
            if !first_inside && !last_inside {
                continue;
            }
            if !first_inside || !last_inside {
                return false;
            }
            if mode == Mode::Machine && configuration & LOCKED == 0 {
                return true;
            }
            return configuration
                & match reach {
                    Reach::Load => 1,
                    Reach::Store => 2,
                    Reach::Fetch => 4,
                }
                != 0;
        }
        mode == Mode::Machine
    }

    pub(crate) fn configuration(&self) -> u64 {
        u64::from_le_bytes(self.configurations)
    }

    fn refresh(&mut self) {
        for index in 0..ENTRIES {
            let address = self.addresses[index];
            self.ranges[index] =
                match (self.configurations[index] & MATCHING) >> 3 {
                    0 => None,
                    1 => {
                        let low = if index == 0 {
                            0
                        } else {
                            self.addresses[index - 1] << 2
                        };
                        (address << 2 > low).then(|| (low, (address << 2) - 1))
                    }
                    2 => Some((address << 2, (address << 2) | 3)),
                    _ => {
                        let ones = address.trailing_ones();
                        let span = (1_u64 << (ones + 3)) - 1;
                        Some(((address << 2) & !span, (address << 2) | span))
                    }
                };
        }
    }

    pub(crate) fn set_address(&mut self, index: usize, value: u64) {
        let Some(configuration) = self.configurations.get(index) else {
            return;
        };
        let follower = self.configurations.get(index + 1).copied().unwrap_or(0);
        let held_by_follower =
            follower & LOCKED != 0 && follower & MATCHING == TOP_OF_RANGE;
        if configuration & LOCKED != 0 || held_by_follower {
            return;
        }
        self.addresses[index] = value & ADDRESS_BITS;
        self.refresh();
    }

    pub(crate) fn set_configuration(&mut self, value: u64) {
        for (held, mut written) in
            self.configurations.iter_mut().zip(value.to_le_bytes())
        {
            if *held & LOCKED != 0 {
                continue;
            }
            if written & 3 == 2 {
                written &= !3;
            }
            *held = written & 0x9f;
        }
        self.refresh();
    }
}
