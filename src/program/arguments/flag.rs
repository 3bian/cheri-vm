// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub struct Flag {
    pub choices: &'static [&'static str],
    pub conflicts: &'static [&'static str],
    pub default: Option<&'static str>,
    pub help: &'static str,
    pub highest: Option<u64>,
    pub long: &'static str,
    pub repeats: bool,
    pub requires: &'static [&'static str],
    pub short: Option<char>,
    pub value: Option<&'static str>,
}

impl Flag {
    pub const fn choice(
        long: &'static str,
        choices: &'static [&'static str],
        help: &'static str,
    ) -> Self {
        let mut flag = Self::value(long, "", help);
        flag.choices = choices;
        flag
    }

    pub const fn conflicting(
        mut self,
        others: &'static [&'static str],
    ) -> Self {
        self.conflicts = others;
        self
    }

    pub const fn number(
        long: &'static str,
        value: &'static str,
        highest: u64,
        help: &'static str,
    ) -> Self {
        let mut flag = Self::value(long, value, help);
        flag.highest = Some(highest);
        flag
    }

    pub const fn repeated(mut self) -> Self {
        self.repeats = true;
        self
    }

    pub const fn requiring(mut self, others: &'static [&'static str]) -> Self {
        self.requires = others;
        self
    }

    pub const fn shortened(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub fn shown(&self) -> String {
        match self.shown_value() {
            Some(value) => format!("--{} {value}", self.long),
            None => format!("--{}", self.long),
        }
    }

    pub fn shown_value(&self) -> Option<String> {
        let value = self.value?;
        Some(if self.choices.is_empty() {
            value.to_owned()
        } else {
            self.choices.join("|")
        })
    }

    pub const fn switch(long: &'static str, help: &'static str) -> Self {
        Self {
            choices: &[],
            conflicts: &[],
            default: None,
            help,
            highest: None,
            long,
            repeats: false,
            requires: &[],
            short: None,
            value: None,
        }
    }

    pub const fn value(
        long: &'static str,
        value: &'static str,
        help: &'static str,
    ) -> Self {
        let mut flag = Self::switch(long, help);
        flag.value = Some(value);
        flag
    }

    pub const fn with_default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }
}

impl Flag {
    pub(super) fn check(&self, value: &str) -> Result<(), String> {
        if !self.choices.is_empty() && !self.choices.contains(&value) {
            return Err(format!(
                "invalid value '{value}' for '{}'; the values are {}",
                self.shown(),
                self.choices.join(", ")
            ));
        }
        if let Some(highest) = self.highest
            && !value.parse::<u64>().is_ok_and(|number| number <= highest)
        {
            return Err(format!(
                "invalid value '{value}' for '{}'; it is a whole number up \
                 to {highest}",
                self.shown()
            ));
        }
        Ok(())
    }
}
