// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{CommandSpec, Flag, Outcome, Parsed};
use std::collections::BTreeMap;
use std::ffi::OsString;

pub struct Program {
    pub commands: &'static [CommandSpec],
    pub name: &'static str,
    pub options: &'static [Flag],
    pub summary: &'static str,
    pub version: &'static str,
}

impl Program {
    pub fn command(&self, name: &str) -> Option<&'static CommandSpec> {
        self.commands.iter().find(|command| command.name == name)
    }

    pub(super) fn flag(
        &self,
        command: Option<&'static CommandSpec>,
        named: impl Fn(&Flag) -> bool,
    ) -> Option<&'static Flag> {
        command
            .into_iter()
            .flat_map(|command| command.options.iter())
            .chain(self.options)
            .find(|flag| named(flag))
    }

    pub fn flags(
        &'static self,
        command: Option<&'static CommandSpec>,
    ) -> Vec<&'static Flag> {
        let mut flags: Vec<&Flag> = command
            .into_iter()
            .flat_map(|command| command.options.iter())
            .chain(self.options)
            .collect();
        flags.sort_by_key(|flag| flag.long);
        flags
    }

    pub fn parse(
        &'static self,
        arguments: impl IntoIterator<Item = OsString>,
    ) -> Outcome {
        let mut parsed = Parsed {
            command: None,
            positionals: Vec::new(),
            values: BTreeMap::new(),
        };
        let mut arguments = arguments.into_iter();
        let mut only_positionals = false;
        while let Some(argument) = arguments.next() {
            let text = argument.to_str().unwrap_or_default();
            if only_positionals || !text.starts_with('-') || text == "-" {
                match (parsed.command, self.command(text)) {
                    (None, Some(command)) => parsed.command = Some(command),
                    (None, None) => {
                        return Outcome::Refused(
                            format!(
                                "unrecognized command '{}'",
                                argument.to_string_lossy()
                            ),
                            None,
                        );
                    }
                    (Some(_), _) => parsed.positionals.push(argument),
                }
                continue;
            }
            if text == "--" {
                only_positionals = true;
                continue;
            }
            let (name, attached) = match text.split_once('=') {
                Some((name, value)) if name.starts_with("--") => {
                    (name, Some(value.to_owned()))
                }
                _ => (text, None),
            };
            match name {
                "-h" | "--help" => return Outcome::Help(parsed.command),
                "-V" | "--version" if parsed.command.is_none() => {
                    return Outcome::Version;
                }
                _ => {}
            }
            let flag = match name.strip_prefix("--") {
                Some(long) => {
                    self.flag(parsed.command, |flag| flag.long == long)
                }
                None => {
                    let mut letters = name.chars().skip(1);
                    let letter = letters.next();
                    letters.next().is_none().then_some(()).and_then(|()| {
                        self.flag(parsed.command, |flag| flag.short == letter)
                    })
                }
            };
            let Some(flag) = flag else {
                return Outcome::Refused(
                    format!("unexpected argument '{name}' found"),
                    parsed.command,
                );
            };
            let value = match (flag.value, attached) {
                (None, None) => String::new(),
                (None, Some(_)) => {
                    return Outcome::Refused(
                        format!("'--{}' takes no value", flag.long),
                        parsed.command,
                    );
                }
                (Some(_), Some(value)) => value,
                (Some(_), None) => {
                    match arguments.next().map(OsString::into_string) {
                        Some(Ok(value)) => value,
                        Some(Err(_)) => {
                            return Outcome::Refused(
                                format!(
                                    "the value of '{}' is not UTF-8",
                                    flag.shown()
                                ),
                                parsed.command,
                            );
                        }
                        None => {
                            return Outcome::Refused(
                                format!(
                                    "a value is required for '{}' but none \
                                     was supplied",
                                    flag.shown()
                                ),
                                parsed.command,
                            );
                        }
                    }
                }
            };
            if let Err(refusal) = flag.check(&value) {
                return Outcome::Refused(refusal, parsed.command);
            }
            let given = parsed.values.entry(flag.long).or_default();
            if !given.is_empty() && !flag.repeats {
                return Outcome::Refused(
                    format!(
                        "the argument '{}' cannot be used multiple times",
                        flag.shown()
                    ),
                    parsed.command,
                );
            }
            given.push(value);
        }
        match self.settled(&parsed) {
            Ok(()) if parsed.command.is_none() => Outcome::Help(None),
            Ok(()) => Outcome::Run(parsed),
            Err(refusal) => Outcome::Refused(refusal, parsed.command),
        }
    }

    pub(super) fn settled(
        &'static self,
        parsed: &Parsed,
    ) -> Result<(), String> {
        let Some(command) = parsed.command else {
            return Ok(());
        };
        if let Some(extra) = parsed.positionals.get(command.positionals.len()) {
            return Err(format!(
                "unexpected argument '{}' found",
                extra.to_string_lossy()
            ));
        }
        for flag in self.flags(parsed.command) {
            if !parsed.has(flag.long) {
                continue;
            }
            if let Some(other) = flag
                .requires
                .iter()
                .find(|other| !parsed.has(other))
                .and_then(|other| command.flag(other))
            {
                return Err(format!(
                    "'{}' needs '{}' as well",
                    flag.shown(),
                    other.shown()
                ));
            }
            if let Some(other) =
                flag.conflicts.iter().find(|other| parsed.has(other))
            {
                return Err(format!(
                    "the argument '{}' cannot be used with '--{other}'",
                    flag.shown()
                ));
            }
        }
        for (index, positional) in command.positionals.iter().enumerate() {
            let given = parsed.positionals.get(index).is_some();
            match positional.unless {
                Some(other) if given && parsed.has(other) => {
                    return Err(format!(
                        "the argument '--{other}' cannot be used with '{}'",
                        positional.name
                    ));
                }
                Some(other) if !given && !parsed.has(other) => {
                    return Err(format!(
                        "the following required argument was not provided: \
                         {}",
                        positional.name
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }
}
