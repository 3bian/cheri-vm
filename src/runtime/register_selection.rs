// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::ExtensionSet;
use crate::machine::EmulatorError;

use crate::runtime::register::Register;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RegisterSelection {
    Names(Vec<Register>),
}

impl RegisterSelection {
    pub(crate) fn includes(&self, register: Register) -> bool {
        let Self::Names(names) = self;
        names.contains(&register)
    }

    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let mut names = Vec::new();
        for name in value.split('+') {
            if names.len() == 64 {
                return Err(EmulatorError::Configuration(
                    "at most 64 register names are accepted".into(),
                ));
            }
            if name.is_empty() {
                return Err(EmulatorError::Configuration(
                    "register name is empty; join names with +, as in pc+x1"
                        .into(),
                ));
            }
            let register = Register::parse(name)?;
            if names.contains(&register) {
                return Err(EmulatorError::Configuration(format!(
                    "duplicate register: {name}"
                )));
            }
            names.push(register);
        }
        Ok(Self::Names(names))
    }

    pub(crate) fn validate(
        &self,
        cheriot: bool,
        extensions: ExtensionSet,
    ) -> Result<(), EmulatorError> {
        let Self::Names(names) = self;
        for name in names {
            name.validate(cheriot, extensions)?;
        }
        Ok(())
    }
}
