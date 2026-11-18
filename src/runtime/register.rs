// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::ExtensionSet;
use crate::machine::EmulatorError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Register {
    Capability(u8),
    Float(u8),
    FloatControl,
    Integer(u8),
    Mepcc,
    Mscratchc,
    Mtcc,
    Mtdc,
    Pc,
    Pcc,
}

impl Register {
    pub(crate) fn parse(name: &str) -> Result<Self, EmulatorError> {
        let selection = match name {
            "fcsr" => Self::FloatControl,
            "mepcc" => Self::Mepcc,
            "mscratchc" => Self::Mscratchc,
            "mtcc" => Self::Mtcc,
            "mtdc" => Self::Mtdc,
            "pc" => Self::Pc,
            "pcc" => Self::Pcc,
            _ => {
                if let Some(index) = name
                    .strip_prefix('x')
                    .and_then(|value| value.parse::<u8>().ok())
                    .filter(|index| *index < 32)
                {
                    return Ok(Self::Integer(index));
                }
                if let Some(index) = name
                    .strip_prefix('f')
                    .and_then(|value| value.parse::<u8>().ok())
                    .filter(|index| *index < 32)
                {
                    return Ok(Self::Float(index));
                }
                if let Some(index) = name
                    .strip_prefix('c')
                    .and_then(|value| value.parse::<u8>().ok())
                    .filter(|index| *index < 16)
                {
                    return Ok(Self::Capability(index));
                }
                return Err(EmulatorError::Configuration(format!(
                    "unknown register: {name}; use x0..x31, f0..f31, fcsr, \
                     pc, or a capability register"
                )));
            }
        };
        Ok(selection)
    }

    pub(crate) fn validate(
        self,
        cheriot: bool,
        extensions: ExtensionSet,
    ) -> Result<(), EmulatorError> {
        let float = !cheriot && extensions.contains(ExtensionSet::F);
        let capabilities = extensions.contains(ExtensionSet::Y);
        let valid = match self {
            Self::Pc => true,
            Self::Integer(index) => {
                !cheriot && usize::from(index) < extensions.register_count()
            }
            Self::Float(_) | Self::FloatControl => float,
            Self::Pcc => cheriot || capabilities,
            _ => cheriot,
        };
        if !valid {
            return Err(EmulatorError::Configuration(format!(
                "requested register is unavailable for this processor; use {}",
                if cheriot {
                    "c0..c15, pc, pcc, mtcc, mtdc, mscratchc or mepcc"
                        .to_owned()
                } else {
                    format!(
                        "x0..x{}{}{} or pc",
                        extensions.register_count() - 1,
                        if float { ", f0..f31, fcsr" } else { "" },
                        if capabilities { ", pcc" } else { "" }
                    )
                }
            )));
        }
        Ok(())
    }
}
