// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

const ALLOWED: [&str; 9] = [
    "gdbstub",
    "gimli",
    "gpiocdev",
    "i2cdev",
    "nix",
    "serde",
    "serde_json",
    "spidev",
    "toml",
];
const DOCUMENTS: [&str; 2] = ["README.md", "THIRD_PARTY_NOTICES.md"];
const HEADER: &str = "// Copyright 3bian Limited and Contributors\n// \
                      SPDX-License-Identifier: MIT\n";
const MEMBERS_KEPT_IN_PLACE: [&str; 3] = [
    "src/isa/pointer/cheriot/capability.rs",
    "src/isa/pointer/rvy/capability.rs",
    "src/machine/host/uart/settings.rs",
];
const OUTSIDE_THE_CORE: [&str; 16] = [
    "crate::debug",
    "crate::profile",
    "crate::program",
    "crate::runtime",
    "crate::web",
    "crate::websocket",
    "gdbstub::",
    "gimli::",
    "gpiocdev::",
    "i2cdev::",
    "nix::",
    "serde::",
    "serde_json::",
    "spidev::",
    "toml::",
    "tun::",
];

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Key(u8, String, String, u8, String, String, String);

fn after_generics(text: &str) -> &str {
    let text = text.trim_start();
    if !text.starts_with('<') {
        return text;
    }
    let mut depth = 0;
    let mut before = ' ';
    for (index, letter) in text.char_indices() {
        match letter {
            '<' => depth += 1,
            '>' if before != '-' => {
                depth -= 1;
                if depth == 0 {
                    return text[index + 1..].trim_start();
                }
            }
            _ => {}
        }
        before = letter;
    }
    text
}

fn files(folder: &str) -> Vec<PathBuf> {
    fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(folder).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name != "target") {
                    walk(&path, found);
                }
            } else {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root().join(folder), &mut found);
    found.sort();
    found
}

fn impl_parts(header: &str) -> (String, String, String) {
    let rest = after_generics(
        header
            .trim()
            .trim_start_matches("unsafe ")
            .trim_start_matches("impl"),
    );
    let rest = rest.split(" where ").next().unwrap_or(rest).trim();
    let rest = rest.split('{').next().unwrap_or(rest).trim();
    let mut depth = 0;
    let mut cut = None;
    let mut before = ' ';
    for (index, letter) in rest.char_indices() {
        match letter {
            '<' | '(' => depth += 1,
            ')' => depth -= 1,
            '>' if before != '-' => depth -= 1,
            _ if depth == 0 && rest[index..].starts_with(" for ") => {
                cut = Some(index);
            }
            _ => {}
        }
        before = letter;
    }
    let (implemented, target) = match cut {
        Some(cut) => (rest[..cut].trim(), rest[cut + 5..].trim()),
        None => ("", rest),
    };
    let mut base = target;
    loop {
        let trimmed = base
            .trim_start_matches('&')
            .trim_start_matches("mut ")
            .trim_start_matches("dyn ")
            .trim_start_matches('(')
            .trim_start();
        if trimmed == base {
            break;
        }
        base = trimmed;
    }
    let path = base.split(['<', ' ', '(', ',']).next().unwrap_or_default();
    let name = path.rsplit("::").next().unwrap_or_default();
    (name.to_owned(), target.to_owned(), implemented.to_owned())
}

fn item(line: &str) -> Option<(&'static str, String)> {
    let mut rest = line;
    if let Some(public) = rest.strip_prefix("pub") {
        rest = match public.strip_prefix('(') {
            Some(scope) => scope.split_once(')')?.1,
            None => public,
        }
        .strip_prefix(' ')?;
    }
    let name = |text: &str| -> String {
        text.chars()
            .take_while(|letter| letter.is_alphanumeric() || *letter == '_')
            .collect()
    };
    for kind in ["struct ", "enum ", "union ", "trait "] {
        if let Some(named) = rest.strip_prefix(kind) {
            return Some(("type", name(named)));
        }
    }
    if let Some(named) = rest.strip_prefix("type ") {
        return Some(("alias", name(named)));
    }
    if rest.starts_with("impl") || rest.starts_with("unsafe impl") {
        return Some(("impl", String::new()));
    }
    for word in ["default ", "const ", "async ", "unsafe ", "extern \"C\" "] {
        if let Some(function) = rest.strip_prefix(word)
            && (function.starts_with("fn ")
                || function.starts_with("unsafe fn ")
                || function.starts_with("async fn "))
        {
            rest = function;
        }
    }
    let rest = rest.strip_prefix("unsafe ").unwrap_or(rest);
    if let Some(function) = rest.strip_prefix("fn ") {
        return Some(("fn", name(function)));
    }
    for kind in ["const ", "static "] {
        if let Some(named) = rest.strip_prefix(kind) {
            let named = named.strip_prefix("mut ").unwrap_or(named);
            return Some(("const", name(named)));
        }
    }
    None
}

fn low(name: &str) -> (String, String) {
    (name.to_lowercase(), name.to_owned())
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust(folder: &str) -> Vec<(String, String)> {
    files(folder)
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ending| ending == "rs"))
        .map(|path| {
            let name = path
                .strip_prefix(root())
                .unwrap()
                .to_string_lossy()
                .into_owned();
            (name, std::fs::read_to_string(&path).unwrap())
        })
        .collect()
}

#[test]
fn a_source_file_holds_at_most_one_type() {
    let mut several = Vec::new();
    for (name, text) in rust("src") {
        let mut types: Vec<String> = text
            .lines()
            .filter_map(item)
            .filter(|(kind, _)| *kind == "type")
            .map(|(_, name)| name)
            .collect();
        types.dedup();
        if types.len() > 1 {
            several.push(format!("{name}: {}", types.join(", ")));
        }
    }
    assert!(several.is_empty(), "{}", several.join("\n"));
}

#[test]
fn every_rust_file_has_the_header_and_no_other_comment() {
    let mut wrong = Vec::new();
    for (name, text) in rust("src").into_iter().chain(rust("tests")) {
        if name.contains("/fixtures/") {
            continue;
        }
        if !text.starts_with(HEADER) {
            wrong.push(format!("{name}: no header"));
        }
        let comments = text
            .lines()
            .skip(2)
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("//") || line.starts_with("/*")
            })
            .count();
        if comments != 0 {
            wrong.push(format!("{name}: {comments} comment lines"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn items_are_in_order_constants_then_types_with_their_impls_then_functions() {
    let mut wrong = Vec::new();
    for (name, text) in rust("src") {
        let lines: Vec<&str> = text.lines().collect();
        let mut keys = Vec::new();
        let mut members: Vec<(u8, (String, String))> = Vec::new();
        let mut ordered = true;
        for (index, line) in lines.iter().enumerate() {
            if let Some(member) = line.strip_prefix("    ")
                && !member.starts_with(' ')
                && let Some((kind, named)) = item(member)
            {
                let rank = u8::from(kind == "fn");
                if kind == "impl" {
                    continue;
                }
                let key = (rank, low(&named));
                ordered &= members.last().is_none_or(|last| *last <= key);
                members.push(key);
            }
            if line.starts_with('}') {
                members.clear();
            }
            let Some((kind, named)) = item(line) else {
                continue;
            };
            members.clear();
            let empty = || (String::new(), String::new());
            keys.push(match kind {
                "const" => {
                    let (lower, exact) = low(&named);
                    Key(
                        0,
                        lower,
                        exact,
                        0,
                        String::new(),
                        String::new(),
                        String::new(),
                    )
                }
                "type" | "alias" => {
                    let (lower, exact) = low(&named);
                    Key(
                        1,
                        lower,
                        exact,
                        0,
                        String::new(),
                        String::new(),
                        String::new(),
                    )
                }
                "impl" => {
                    let mut header = String::new();
                    for more in &lines[index..] {
                        header.push_str(more.trim());
                        header.push(' ');
                        if more.contains('{') {
                            break;
                        }
                    }
                    let (target, whole, implemented) = impl_parts(&header);
                    let (lower, exact) = low(&target);
                    let (implemented_lower, implemented_exact) =
                        if implemented.is_empty() {
                            empty()
                        } else {
                            low(&implemented)
                        };
                    Key(
                        1,
                        lower,
                        exact,
                        if implemented.is_empty() { 1 } else { 2 },
                        implemented_lower,
                        implemented_exact,
                        whole,
                    )
                }
                _ => {
                    let (lower, exact) = low(&named);
                    Key(
                        2,
                        lower,
                        exact,
                        0,
                        String::new(),
                        String::new(),
                        String::new(),
                    )
                }
            });
        }
        if let Some(pair) = keys.windows(2).find(|pair| pair[0] > pair[1]) {
            wrong.push(format!(
                "{name}: {} {} comes before {} {}",
                pair[0].2, pair[0].5, pair[1].2, pair[1].5
            ));
        }
        if !ordered && !MEMBERS_KEPT_IN_PLACE.contains(&name.as_str()) {
            wrong.push(format!("{name}: the members of an impl"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn no_source_file_is_over_500_lines() {
    let long: Vec<String> = rust("src")
        .into_iter()
        .filter(|(_, text)| text.lines().count() > 500)
        .map(|(name, text)| format!("{name}: {}", text.lines().count()))
        .collect();
    assert!(long.is_empty(), "{}", long.join("\n"));
}

#[test]
fn the_core_names_nothing_outside_itself() {
    let mut reaching = Vec::new();
    for folder in ["src/isa", "src/hart", "src/machine"] {
        for (name, text) in rust(folder) {
            let apart = name.starts_with("src/machine/host")
                || (name.starts_with("src/hart/")
                    && name.split('/').skip(3).any(|part| {
                        ["debug", "debug.rs", "present.rs"].contains(&part)
                    }));
            if apart {
                continue;
            }
            for outside in OUTSIDE_THE_CORE {
                if text.contains(outside) {
                    reaching.push(format!("{name}: {outside}"));
                }
            }
        }
    }
    assert!(reaching.is_empty(), "{}", reaching.join("\n"));
}

#[test]
fn the_program_depends_on_the_packages_it_is_allowed_and_no_others() {
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).unwrap();
    let mut named = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        if line.starts_with('[') {
            inside =
                line.ends_with("dependencies]") && !line.starts_with("[dev-");
            continue;
        }
        if inside
            && let Some((name, _)) = line.split_once(" = ")
            && !name.starts_with(' ')
        {
            named.push(name.to_owned());
        }
    }
    named.sort();
    let mut allowed: Vec<&str> = ALLOWED.to_vec();
    allowed.push("tun");
    allowed.sort_unstable();
    assert_eq!(named, allowed);
}

#[test]
fn the_repository_holds_no_document_but_its_readme_and_notices() {
    let documents: Vec<String> = files("")
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ending| ending == "md"))
        .map(|path| {
            path.strip_prefix(root())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| !name.starts_with("tests/fixtures/"))
        .collect();
    assert_eq!(documents, DOCUMENTS);
}
