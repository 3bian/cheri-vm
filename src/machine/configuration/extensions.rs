// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e;
use crate::isa::extension::{EXTENSIONS, ExtensionSet};
use crate::machine::error::EmulatorError;

pub(super) fn check_extensions(
    processor: &str,
    extensions: ExtensionSet,
) -> Result<(), EmulatorError> {
    let invalid = |message: String| Err(EmulatorError::Configuration(message));
    match processor {
        "cheriot" if extensions != cheriot_rv32e::Processor::EXTENSIONS => {
            invalid(format!(
                "a cheriot processor has one set of extensions: {}; leave \
                 extensions out, or list exactly these",
                cheriot_rv32e::Processor::EXTENSIONS
                    .names()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
        "rv32" | "rv64" if extensions.contains(ExtensionSet::CHERIOT) => {
            invalid(
                "the cheriot extension needs processor = \"cheriot\"".into(),
            )
        }
        "rv32"
            if extensions.contains(ExtensionSet::I)
                == extensions.contains(ExtensionSet::E) =>
        {
            invalid(
                "extensions must list one of i and e, the base instructions \
                 with 32 or 16 registers"
                    .into(),
            )
        }
        "rv32" | "rv64"
            if extensions.contains(ExtensionSet::D)
                && !extensions.contains(ExtensionSet::F) =>
        {
            invalid(
                "d needs f: a machine with double precision has single \
                 precision too"
                    .into(),
            )
        }
        "rv32" | "rv64"
            if extensions.contains(ExtensionSet::F)
                && !(extensions.contains(ExtensionSet::I)
                    && extensions.contains(ExtensionSet::ZICSR)) =>
        {
            invalid(
                "f needs i and zicsr: 32 registers, and the register its \
                 flags are read through"
                    .into(),
            )
        }
        "rv32" | "rv64"
            if extensions.contains(ExtensionSet::ZCB)
                && !extensions.contains(
                    ExtensionSet::C
                        .with(ExtensionSet::M)
                        .with(ExtensionSet::ZBA)
                        .with(ExtensionSet::ZBB),
                ) =>
        {
            invalid(
                "zcb needs c, m, zba and zbb: its 16-bit forms stand for \
                 instructions of all four"
                    .into(),
            )
        }
        "rv32" if extensions.contains(ExtensionSet::Y) => invalid(
            "y needs processor = \"rv64\": the capability format built is \
             RV64LYA"
                .into(),
        ),
        "rv64"
            if extensions.contains(ExtensionSet::Y)
                && !extensions.contains(ExtensionSet::ZICSR) =>
        {
            invalid(
                "y needs zicsr: a capability machine gets its first \
                 capability from mtvec"
                    .into(),
            )
        }
        "rv32" | "rv64"
            if extensions.contains(ExtensionSet::ZYLG)
                && !extensions.contains(ExtensionSet::Y) =>
        {
            invalid("zylg needs y: it adds a level to an RVY capability".into())
        }
        "rv32" | "rv64"
            if extensions.contains(ExtensionSet::SVYRG)
                && !extensions
                    .contains(ExtensionSet::Y.with(ExtensionSet::S)) =>
        {
            invalid(
                "svyrg needs y and s: it gives meaning to bits of an Sv39 \
                 page-table entry for capabilities"
                    .into(),
            )
        }
        "rv32"
            if extensions.contains(ExtensionSet::S)
                || extensions.contains(ExtensionSet::U) =>
        {
            invalid(
                "s and u need processor = \"rv64\": translation is Sv39, and \
                 there is no Sv32"
                    .into(),
            )
        }
        "rv64"
            if extensions.contains(ExtensionSet::S)
                != extensions.contains(ExtensionSet::U) =>
        {
            invalid(
                "list s and u together: a machine has machine mode alone, or \
                 all three modes"
                    .into(),
            )
        }
        "rv64"
            if extensions.contains(ExtensionSet::S)
                && !extensions.contains(ExtensionSet::ZICSR) =>
        {
            invalid(
                "s and u need zicsr, which has the registers the modes are \
                 entered through"
                    .into(),
            )
        }
        "rv64"
            if !extensions.contains(ExtensionSet::I)
                || extensions.contains(ExtensionSet::E) =>
        {
            invalid(
                "an rv64 processor has the base instructions with 32 \
                 registers: list i, and not e"
                    .into(),
            )
        }
        "cheriot" | "rv32" | "rv64" => Ok(()),
        other if other.starts_with("rv32") || other.starts_with("rv64") => {
            invalid(format!(
                "unsupported processor type: {other}; write processor = \
                 \"{}\" and list its letters in extensions",
                &other[..4]
            ))
        }
        other => invalid(format!(
            "unsupported processor type: {other}; the processors are cheriot, \
             rv32 and rv64"
        )),
    }
}

pub(super) fn extensions_for(
    processor: &str,
    listed: Option<&[String]>,
) -> Result<ExtensionSet, EmulatorError> {
    let known = || {
        EXTENSIONS
            .iter()
            .map(|extension| extension.name)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let extensions = match (processor, listed) {
        ("cheriot", None) => cheriot_rv32e::Processor::EXTENSIONS,
        (_, Some(names)) => ExtensionSet::named(
            names.iter().map(String::as_str),
        )
        .map_err(|name| {
            EmulatorError::Configuration(format!(
                "extension {name} is unknown or listed twice; the extensions \
                 are {}",
                known()
            ))
        })?,
        ("rv32" | "rv64", None) => {
            return Err(EmulatorError::Configuration(format!(
                "an {processor} processor needs an extensions list, such as \
                 extensions = [\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\"]; \
                 the extensions are {}",
                known()
            )));
        }
        _ => ExtensionSet::default(),
    };
    check_extensions(processor, extensions)?;
    Ok(extensions)
}
