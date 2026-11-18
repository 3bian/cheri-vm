// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::ProfileDocument;
use super::built_in;
use crate::program::diagnostic::{Diagnostic, Result};
use crate::program::{output, settings};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MACHINE_KEYS: &[&str] = &[
    "attached_device",
    "interrupt",
    "interrupt_source",
    "read_only",
    "sectors",
];

const MAX_BYTES: u64 = 1024 * 1024;

struct Loaded {
    file: PathBuf,
    table: toml::Table,
}

fn checked(table: &toml::Table, file: &Path) -> Result<()> {
    let fail = |message: String| {
        Diagnostic::new(format!("{}: {message}", file.display()))
    };
    let devices = match table.get("device") {
        Some(toml::Value::Table(devices)) => devices,
        Some(_) => return Err(fail("device must be a table".into())),
        None => return Ok(()),
    };
    for (name, device) in devices {
        let Some(device) = device.as_table() else {
            return Err(fail(format!("device {name} must be a table")));
        };
        if !device.contains_key("model")
            && MACHINE_KEYS.iter().any(|key| device.contains_key(*key))
        {
            return Err(fail(format!(
                "device {name} has emulator keys but no model"
            )));
        }
        if let Some(interrupt) =
            device.get("interrupt").and_then(toml::Value::as_str)
            && table
                .get("interrupt")
                .and_then(|interrupts| interrupts.get(interrupt))
                .is_none()
        {
            return Err(fail(format!(
                "device {name} names the interrupt {interrupt}, which \
                 [interrupt] does not define"
            )));
        }
    }
    Ok(())
}

fn finish(
    mut table: toml::Table,
    file: &Path,
    directory: &Path,
    stack: &mut BTreeSet<PathBuf>,
) -> Result<toml::Table> {
    let fail = |message: String| {
        Diagnostic::new(format!("{}: {message}", file.display()))
    };
    let parent = table.remove("extends");
    let removals = table.remove("remove");
    let mut resolved = match parent {
        Some(toml::Value::String(name)) if is_profile_name(&name) => {
            let mut base = load_named(&name, Some(directory), stack)?.table;
            merge(&mut base, table);
            base
        }
        Some(other) => {
            return Err(fail(format!(
                "extends takes a profile name such as \"cheriot\", not {other}"
            )));
        }
        None => table,
    };
    if let Some(removals) = removals {
        remove_paths(&mut resolved, &removals).map_err(fail)?;
    }
    Ok(resolved)
}

fn is_profile_name(value: &str) -> bool {
    !value.contains(['/', '\\']) && !value.ends_with(".toml")
}

pub fn load(selector: &str) -> Result<ProfileDocument> {
    let mut stack = BTreeSet::new();
    let loaded = if is_profile_name(selector) {
        load_named(selector, None, &mut stack)?
    } else {
        let file = Path::new(selector);
        if !file.is_file() {
            return Err(Diagnostic::new(format!(
                "profile file {} does not exist{}",
                file.display(),
                output::choices("Profiles", &names())
            )));
        }
        load_file(file, &mut stack)?
    };
    checked(&loaded.table, &loaded.file)?;
    let text = |key: &str| {
        loaded
            .table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let name = text("name").ok_or_else(|| {
        Diagnostic::new(format!(
            "{}: a profile needs a name",
            loaded.file.display()
        ))
    })?;
    Ok(ProfileDocument {
        description: text("description"),
        file: loaded.file,
        name,
        table: loaded.table,
    })
}

fn load_built_in(
    name: &str,
    text: &str,
    stack: &mut BTreeSet<PathBuf>,
) -> Result<Loaded> {
    let file = built_in::path(name);
    if !stack.insert(file.clone()) {
        return Err(Diagnostic::new(format!(
            "profile inheritance cycle at {}",
            file.display()
        )));
    }
    let table = parsed(&file, text)?;
    let table = finish(table, &file, Path::new(built_in::FOLDER), stack)?;
    stack.remove(&file);
    Ok(Loaded { file, table })
}

fn load_file(file: &Path, stack: &mut BTreeSet<PathBuf>) -> Result<Loaded> {
    let file = std::fs::canonicalize(file)
        .map_err(|error| Diagnostic::io(file, error))?;
    if !stack.insert(file.clone()) {
        return Err(Diagnostic::new(format!(
            "profile inheritance cycle at {}",
            file.display()
        )));
    }
    let length = std::fs::metadata(&file)
        .map_err(|error| Diagnostic::io(&file, error))?
        .len();
    if length > MAX_BYTES {
        return Err(Diagnostic::io(&file, "a profile is at most 1 MiB"));
    }
    let text = std::fs::read_to_string(&file)
        .map_err(|error| Diagnostic::io(&file, error))?;
    let table = parsed(&file, &text)?;
    let directory = file
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| Diagnostic::new("profile file needs a directory"))?;
    let table = finish(table, &file, &directory, stack)?;
    stack.remove(&file);
    Ok(Loaded { file, table })
}

fn load_named(
    name: &str,
    sibling_directory: Option<&Path>,
    stack: &mut BTreeSet<PathBuf>,
) -> Result<Loaded> {
    let built_in_sibling =
        sibling_directory == Some(Path::new(built_in::FOLDER));
    if built_in_sibling && let Some(text) = built_in::text(name) {
        return load_built_in(name, text, stack);
    }
    let own = settings::profiles();
    let folders = sibling_directory
        .filter(|_| !built_in_sibling)
        .into_iter()
        .chain(own.as_deref());
    for folder in folders {
        let file = folder.join(format!("{name}.toml"));
        if file.is_file() {
            return load_file(&file, stack);
        }
    }
    match built_in::text(name) {
        Some(text) => load_built_in(name, text, stack),
        None => Err(Diagnostic::new(format!(
            "unknown profile {name:?}: there is no {name}.toml in {}, and no \
             built-in profile of that name{}",
            own.map_or_else(
                || "a profiles folder".to_owned(),
                |folder| folder.display().to_string()
            ),
            output::choices("Profiles", &names())
        ))),
    }
}

fn merge(base: &mut toml::Table, over: toml::Table) {
    for (key, value) in over {
        match (base.get_mut(&key), value) {
            (Some(toml::Value::Table(existing)), toml::Value::Table(value)) => {
                merge(existing, value);
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

pub fn names() -> Vec<String> {
    let mut names = built_in::names();
    if let Some(entries) =
        settings::profiles().and_then(|folder| std::fs::read_dir(folder).ok())
    {
        names.extend(entries.flatten().filter_map(|entry| {
            let path = entry.path();
            (path.extension()? == "toml")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))?
        }));
    }
    names.sort();
    names.dedup();
    names
}

fn parsed(file: &Path, text: &str) -> Result<toml::Table> {
    text.parse::<toml::Table>().map_err(|error| {
        Diagnostic::io(file, crate::program::wording::plain(&error.to_string()))
    })
}

fn remove_paths(
    table: &mut toml::Table,
    removals: &toml::Value,
) -> std::result::Result<(), String> {
    let paths = removals
        .as_array()
        .ok_or_else(|| "remove must be a list of key paths".to_owned())?;
    for path in paths {
        let text = path
            .as_str()
            .ok_or_else(|| "remove must be a list of key paths".to_owned())?;
        let missing = || format!("remove names {text}, which is not present");
        let parts: Vec<&str> = text.split('/').collect();
        let (last, parents) = parts.split_last().ok_or_else(missing)?;
        let mut current = &mut *table;
        for part in parents {
            current = current
                .get_mut(*part)
                .and_then(toml::Value::as_table_mut)
                .ok_or_else(missing)?;
        }
        current.remove(*last).ok_or_else(missing)?;
    }
    Ok(())
}
