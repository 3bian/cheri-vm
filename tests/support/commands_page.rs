// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::help::update_snapshots;
use super::programs::repository;
use cheri_vm::program::arguments::{CommandSpec, Program};
use cheri_vm::program::help::{spelled, usage};

const BOOK: &str = "../book";
const COMMANDS_PAGE: &str = "../book/appendix-a-commands.md";

const INTRODUCTION: &str = "# Commands

Every command of `cheri-vm`, and every option of each.
The tests write this page from the program itself,
and fail when they differ: run them with `CHERI_VM_TEST_UPDATE_SNAPSHOTS=1`
to write it again, and do not edit it by hand.
";

pub fn assert_commands_page(command: &'static Program) {
    let program = command.name;
    let repository = repository();
    if !repository.join(BOOK).is_dir() {
        return;
    }
    let file = repository.join(COMMANDS_PAGE);
    let written = format!("\n{}", program_section(command));
    let page = match std::fs::read_to_string(&file) {
        Ok(page) => page,
        Err(_) if update_snapshots() => skeleton(),
        Err(error) => panic!("{COMMANDS_PAGE} cannot be read: {error}"),
    };
    let (from, to) = between(&page, program)
        .unwrap_or_else(|| panic!("{COMMANDS_PAGE} has no part for {program}"));
    if update_snapshots() {
        std::fs::write(
            &file,
            format!("{}{written}{}", &page[..from], &page[to..]),
        )
        .unwrap();
        return;
    }
    assert_eq!(
        &page[from..to],
        written,
        "{COMMANDS_PAGE} differs from {program}; run the tests with \
         CHERI_VM_TEST_UPDATE_SNAPSHOTS=1 to write it again"
    );
}

fn between(page: &str, program: &str) -> Option<(usize, usize)> {
    let from = page.find(&start(program))? + start(program).len();
    let to = page[from..].find(&end(program))? + from;
    Some((from, to))
}

fn cell(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn code(text: &str) -> String {
    format!("`{}`", text.replace('|', "\\|"))
}

fn end(program: &str) -> String {
    format!("<!-- {program} end -->")
}

fn program_section(program: &'static Program) -> String {
    let mut page = String::new();
    section(&mut page, program, None);
    let mut commands: Vec<&CommandSpec> = program.commands.iter().collect();
    commands.sort_by_key(|command| command.name);
    for command in commands {
        section(&mut page, program, Some(command));
    }
    page
}

fn section(
    page: &mut String,
    program: &'static Program,
    command: Option<&'static CommandSpec>,
) {
    let path = match command {
        Some(command) => format!("{} {}", program.name, command.name),
        None => program.name.to_owned(),
    };
    page.push_str(&format!("\n## `{path}`\n\n"));
    page.push_str(&format!("Usage: `{}`\n", usage(program, command).join(" ")));
    let summary = command.map_or(program.summary, |command| command.summary);
    page.push_str(&format!("\n{}.\n", cell(summary)));
    let arguments = command.map_or(&[][..], |command| command.positionals);
    if !arguments.is_empty() {
        page.push_str("\n| Argument | Help |\n| --- | --- |\n");
        for argument in arguments {
            page.push_str(&format!(
                "| {} | {} |\n",
                code(argument.name),
                cell(argument.help)
            ));
        }
    }
    page.push_str(
        "\n| Option | Value | Default | Repeats | Conflicts with | Help |\n| \
         --- | --- | --- | --- | --- | --- |\n",
    );
    for flag in program.flags(command) {
        let conflicts: Vec<String> = flag
            .conflicts
            .iter()
            .map(|other| format!("`--{other}`"))
            .collect();
        page.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            code(&spelled(flag)),
            flag.shown_value()
                .map(|value| code(&value))
                .unwrap_or_default(),
            flag.default.map(code).unwrap_or_default(),
            if flag.repeats { "yes" } else { "" },
            conflicts.join(", "),
            cell(flag.help)
        ));
    }
}

fn skeleton() -> String {
    let mut page = String::from(INTRODUCTION);
    let program = super::documented::PROGRAM;
    page.push_str(&format!("\n{}\n{}\n", start(program), end(program)));
    page
}

fn start(program: &str) -> String {
    format!("<!-- {program} start -->")
}
