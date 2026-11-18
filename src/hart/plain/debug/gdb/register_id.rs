// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use core::marker::PhantomData;
use std::num::NonZeroUsize;

use gdbstub::arch::RegId;

use super::Width;

pub(crate) const CSR_ADDRESSES: [u16; 26] = [
    0x300, 0x301, 0x304, 0x305, 0x310, 0x320, 0x340, 0x341, 0x342, 0x343,
    0x344, 0xb00, 0xb02, 0xb80, 0xb82, 0xc00, 0xc01, 0xc02, 0xc80, 0xc81,
    0xc82, 0xf11, 0xf12, 0xf13, 0xf14, 0xf15,
];

pub(crate) const FIRST_CSR: usize = 65;
pub(crate) const FIRST_FLOAT: usize = 33;
pub(crate) const PRIVILEGE: usize = 4161;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegisterId<X>(usize, PhantomData<X>);

impl<X> RegisterId<X> {
    pub(crate) fn raw(self) -> usize {
        self.0
    }
}

impl<X: Width> RegId for RegisterId<X> {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        let word = (X::BITS / 8) as usize;
        let size = match id {
            0..FIRST_FLOAT => word,
            _ if X::BITS == 32 => {
                CSR_ADDRESSES.iter().position(|address| {
                    usize::from(*address) + FIRST_CSR == id
                })?;
                word
            }
            FIRST_FLOAT..=64 => 0,
            66..=68 => 4,
            65..=PRIVILEGE => word,
            _ => return None,
        };
        Some((Self(id, PhantomData), NonZeroUsize::new(size)))
    }

    fn to_raw_id(&self) -> Option<usize> {
        Some(self.0)
    }
}
