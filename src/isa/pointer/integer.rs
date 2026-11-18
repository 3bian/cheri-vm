// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use core::marker::PhantomData;

use super::{Access, PointerModel, Violation};
use crate::isa::xlen::Word;

pub struct Integer<W>(PhantomData<W>);

impl<W: Word> PointerModel for Integer<W> {
    type Address = W;
    type Fetch = W;
    type Register = W;

    fn address(register: W) -> W {
        register
    }

    fn check_access(
        _: W,
        address: W,
        width: usize,
        _: Access,
    ) -> Result<(), Violation> {
        if address.to_u64() & (width as u64 - 1) != 0 {
            return Err(Violation::Alignment);
        }
        Ok(())
    }

    #[inline]
    fn check_fetch(_: W, _: W) -> Result<(), Violation> {
        Ok(())
    }
}
