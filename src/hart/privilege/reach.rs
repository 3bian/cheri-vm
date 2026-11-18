// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::ExceptionCause;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reach {
    Fetch,
    Load,
    Store,
}

impl Reach {
    pub(crate) fn access_fault(self) -> ExceptionCause {
        match self {
            Self::Fetch => ExceptionCause::InstructionAccessFault,
            Self::Load => ExceptionCause::LoadAccessFault,
            Self::Store => ExceptionCause::StoreAccessFault,
        }
    }

    pub(crate) fn page_fault(self) -> ExceptionCause {
        match self {
            Self::Fetch => ExceptionCause::InstructionPageFault,
            Self::Load => ExceptionCause::LoadPageFault,
            Self::Store => ExceptionCause::StorePageFault,
        }
    }
}
