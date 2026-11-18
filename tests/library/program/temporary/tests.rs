// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::temporary::{
    TemporaryDirectory, TemporaryFile, create_file,
};
use std::{fs, io::Write};

#[test]
fn a_dropped_directory_is_removed_with_what_it_holds() {
    let folder = TemporaryDirectory::new().unwrap();
    let path = folder.path().to_path_buf();
    fs::create_dir(path.join("inner")).unwrap();
    fs::write(path.join("inner/file"), "text").unwrap();
    drop(folder);
    assert!(!path.exists());
}

#[test]
fn a_dropped_file_is_removed() {
    let folder = TemporaryDirectory::new().unwrap();
    let file =
        TemporaryFile::new_in(folder.path(), ".staged-", ".toml").unwrap();
    let path = file.path().to_path_buf();
    assert!(path.is_file());
    drop(file);
    assert!(!path.exists());
}

#[test]
fn a_name_is_the_prefix_sixteen_digits_and_the_suffix() {
    let folder = TemporaryDirectory::new().unwrap();
    let file =
        TemporaryFile::new_in(folder.path(), ".staged-", ".toml").unwrap();
    let name = file.path().file_name().unwrap().to_str().unwrap();
    let digits = name
        .strip_prefix(".staged-")
        .and_then(|rest| rest.strip_suffix(".toml"))
        .unwrap();
    assert_eq!(digits.len(), 16);
    assert!(digits.chars().all(|digit| digit.is_ascii_hexdigit()));
}

#[test]
fn a_persisted_file_is_kept_under_its_new_name() {
    let folder = TemporaryDirectory::new().unwrap();
    let mut file =
        TemporaryFile::new_in(folder.path(), ".staged-", ".toml").unwrap();
    file.write_all(b"mode = \"debug\"\n").unwrap();
    let staged = file.path().to_path_buf();
    let target = folder.path().join("config.toml");
    file.persist(&target).unwrap();
    assert!(!staged.exists());
    assert_eq!(fs::read_to_string(target).unwrap(), "mode = \"debug\"\n");
}

#[cfg(unix)]
#[test]
fn a_planted_symbolic_link_is_refused_and_not_followed() {
    let folder = TemporaryDirectory::new().unwrap();
    let target = folder.path().join("target");
    let planted = folder.path().join("planted");
    std::os::unix::fs::symlink(&target, &planted).unwrap();
    let error = create_file(&planted).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(!target.exists());
}

#[cfg(unix)]
#[test]
fn the_file_is_private_to_its_owner_and_so_is_the_directory() {
    use std::os::unix::fs::PermissionsExt;
    let folder = TemporaryDirectory::new().unwrap();
    let file = TemporaryFile::new_in(folder.path(), ".staged-", "").unwrap();
    let mode = |path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(file.path()), 0o600);
    assert_eq!(mode(folder.path()), 0o700);
}
