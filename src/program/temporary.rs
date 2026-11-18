// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod temporary_directory;
mod temporary_file;
mod temporary_link;

pub use temporary_directory::TemporaryDirectory;
pub use temporary_file::TemporaryFile;
pub use temporary_link::TemporaryLink;

use std::{
    collections::hash_map::RandomState,
    fs::{self, DirBuilder, File, OpenOptions},
    hash::{BuildHasher, Hasher},
    io::{self},
    path::{Path, PathBuf},
};

const ATTEMPTS: usize = 8;

fn create_directory(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path)
}

pub(crate) fn create_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

pub fn new_folder_in(folder: &Path, prefix: &str) -> io::Result<PathBuf> {
    unused_name(folder, prefix, "", |path| fs::create_dir(path))
        .map(|((), path)| path)
}

fn unused_name<T>(
    folder: &Path,
    prefix: &str,
    suffix: &str,
    create: impl Fn(&Path) -> io::Result<T>,
) -> io::Result<(T, PathBuf)> {
    let mut taken = None;
    for _ in 0..ATTEMPTS {
        let digits = RandomState::new().build_hasher().finish();
        let path = folder.join(format!("{prefix}{digits:016x}{suffix}"));
        match create(&path) {
            Ok(made) => return Ok((made, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                taken = Some(error)
            }
            Err(error) => return Err(error),
        }
    }
    Err(taken.unwrap_or_else(|| io::ErrorKind::AlreadyExists.into()))
}
