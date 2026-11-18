// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::arithmetic::Arithmetic;
use core::fmt::{Debug, LowerHex};
use core::ops::{BitAnd, BitOr, Not};

impl Word for u32 {
    const BITS: u32 = 32;

    fn from_i32(value: i32) -> Self {
        value as u32
    }

    fn from_u64(value: u64) -> Self {
        value as u32
    }

    fn to_i64(self) -> i64 {
        i64::from(self as i32)
    }

    fn to_u64(self) -> u64 {
        u64::from(self)
    }
}

impl Word for u64 {
    const BITS: u32 = 64;

    fn from_i32(value: i32) -> Self {
        i64::from(value) as u64
    }

    fn from_u64(value: u64) -> Self {
        value
    }

    fn to_i64(self) -> i64 {
        self as i64
    }

    fn to_u64(self) -> u64 {
        self
    }
}

pub trait Word:
    Arithmetic
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Copy
    + Debug
    + Default
    + Eq
    + LowerHex
    + Not<Output = Self>
    + Ord
    + 'static
{
    const BITS: u32;

    fn add(self, other: Self) -> Self {
        Self::from_u64(self.to_u64().wrapping_add(other.to_u64()))
    }

    fn add_signed(self, offset: i32) -> Self {
        self.add(Self::from_i32(offset))
    }

    fn from_i32(value: i32) -> Self;

    fn from_u64(value: u64) -> Self;

    fn to_i64(self) -> i64;

    fn to_u64(self) -> u64;
}
