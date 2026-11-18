// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExceptionCause {
    Breakpoint,
    CheriInstructionAccessFault,
    CheriLoadAccessFault,
    CheriLoadCapabilityFault,
    CheriStoreAccessFault,
    CheriStorePageFault,
    EnvironmentCall,
    EnvironmentCallFromSupervisor,
    EnvironmentCallFromUser,
    IllegalInstruction,
    InstructionAccessFault,
    InstructionAddressMisaligned,
    InstructionPageFault,
    LoadAccessFault,
    LoadAddressMisaligned,
    LoadPageFault,
    StoreAccessFault,
    StoreAddressMisaligned,
    StorePageFault,
}

impl ExceptionCause {
    pub(crate) fn code(self) -> u32 {
        match self {
            Self::InstructionAddressMisaligned => 0,
            Self::InstructionAccessFault => 1,
            Self::IllegalInstruction => 2,
            Self::Breakpoint => 3,
            Self::LoadAddressMisaligned => 4,
            Self::LoadAccessFault => 5,
            Self::StoreAddressMisaligned => 6,
            Self::StoreAccessFault => 7,
            Self::EnvironmentCallFromUser => 8,
            Self::EnvironmentCallFromSupervisor => 9,
            Self::EnvironmentCall => 11,
            Self::InstructionPageFault => 12,
            Self::LoadPageFault => 13,
            Self::StorePageFault => 15,
            Self::CheriInstructionAccessFault => 32,
            Self::CheriLoadAccessFault => 33,
            Self::CheriStoreAccessFault => 34,
            Self::CheriLoadCapabilityFault => 35,
            Self::CheriStorePageFault => 36,
        }
    }

    pub(crate) fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::InstructionAddressMisaligned,
            1 => Self::InstructionAccessFault,
            2 => Self::IllegalInstruction,
            3 => Self::Breakpoint,
            4 => Self::LoadAddressMisaligned,
            5 => Self::LoadAccessFault,
            6 => Self::StoreAddressMisaligned,
            7 => Self::StoreAccessFault,
            8 => Self::EnvironmentCallFromUser,
            9 => Self::EnvironmentCallFromSupervisor,
            11 => Self::EnvironmentCall,
            12 => Self::InstructionPageFault,
            13 => Self::LoadPageFault,
            15 => Self::StorePageFault,
            32 => Self::CheriInstructionAccessFault,
            33 => Self::CheriLoadAccessFault,
            34 => Self::CheriStoreAccessFault,
            35 => Self::CheriLoadCapabilityFault,
            36 => Self::CheriStorePageFault,
            _ => return None,
        })
    }
}
