// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::arguments::{CommandSpec, Flag, Program};

const HEADER: &str = "\u{1b}[1;32m";
const LITERAL: &str = "\u{1b}[1;36m";
const PLACEHOLDER: &str = "\u{1b}[36m";
const RESET: &str = "\u{1b}[0m";

fn description(out: &mut String, text: &str) {
    for paragraph in text.lines() {
        wrapped(out, paragraph, "    ");
    }
}

fn header(text: &str) -> String {
    format!("{HEADER}{text}{RESET}")
}

fn literal(text: &str) -> String {
    format!("{LITERAL}{text}{RESET}")
}

fn placeholder(text: &str) -> String {
    format!("{PLACEHOLDER}{text}{RESET}")
}

pub fn render(
    program: &'static Program,
    command: Option<&'static CommandSpec>,
) -> String {
    let words = usage(program, command);
    let mut out = format!("{} {}", header("Usage:"), literal(&words[0]));
    for word in &words[1..] {
        out.push(' ');
        out.push_str(&placeholder(word));
    }
    out.push_str("\n\n");
    let summary = command.map_or(program.summary, |command| command.summary);
    wrapped(&mut out, &sentence(summary), "");
    match command {
        Some(command) if !command.positionals.is_empty() => {
            out.push_str(&format!("\n{}\n", header("Arguments:")));
            for positional in command.positionals {
                out.push_str(&format!("  {}\n", placeholder(positional.name)));
                description(&mut out, &sentence(positional.help));
                out.push('\n');
            }
            out.pop();
        }
        Some(_) => {}
        None => {
            out.push_str(&format!("\n{}\n", header("Commands:")));
            let mut commands: Vec<&CommandSpec> =
                program.commands.iter().collect();
            commands.sort_by_key(|command| command.name);
            for command in commands {
                out.push_str(&format!("  {}\n", literal(command.name)));
                description(&mut out, &sentence(command.summary));
                out.push('\n');
            }
            out.pop();
        }
    }
    out.push_str(&format!("\n{}\n", header("Options:")));
    for flag in program.flags(command) {
        out.push_str("  ");
        if let Some(short) = flag.short {
            out.push_str(&format!("{}, ", literal(&format!("-{short}"))));
        }
        out.push_str(&literal(&format!("--{}", flag.long)));
        if let Some(value) = flag.shown_value() {
            out.push(' ');
            out.push_str(&placeholder(&value));
        }
        out.push('\n');
        description(&mut out, &sentence(flag.help));
        out.push('\n');
    }
    if command.is_none() {
        out.push_str(&format!(
            "  {}, {}\n    Display the version, and exit.\n\n",
            literal("-V"),
            literal("--version")
        ));
    }
    out.push_str(&format!(
        "  {}, {}\n    Display this help and exit.\n",
        literal("-h"),
        literal("--help")
    ));
    out
}

fn sentence(text: &str) -> String {
    let text = text.trim_end();
    if text.ends_with(['.', '?', '!']) {
        text.to_owned()
    } else {
        format!("{text}.")
    }
}

pub fn spelled(flag: &Flag) -> String {
    match flag.short {
        Some(short) => format!("-{short}, --{}", flag.long),
        None => format!("--{}", flag.long),
    }
}

pub fn usage(program: &Program, command: Option<&CommandSpec>) -> Vec<String> {
    match command {
        Some(command) => {
            let mut words = vec![
                format!("{} {}", program.name, command.name),
                "[OPTIONS]".to_owned(),
            ];
            words.extend(
                command
                    .positionals
                    .iter()
                    .map(|positional| format!("[{}]", positional.name)),
            );
            words
        }
        None => vec![
            program.name.to_owned(),
            "[OPTIONS]".to_owned(),
            "[COMMAND]".to_owned(),
        ],
    }
}

fn wrapped(out: &mut String, text: &str, indent: &str) {
    let mut line = String::from(indent);
    for word in text.split_whitespace() {
        if line.len() + word.len() + 1 > 80 && line.len() > indent.len() {
            out.push_str(line.trim_end());
            out.push('\n');
            line = String::from(indent);
        }
        if line.len() > indent.len() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if line.len() > indent.len() {
        out.push_str(&line);
        out.push('\n');
    }
}
