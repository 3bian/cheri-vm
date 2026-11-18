// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{create_file, unused_name};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

pub struct TemporaryFile {
    pub(super) file: File,
    pub(super) path: PathBuf,
    pub(super) kept: bool,
}

impl TemporaryFile {
    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn new_in(
        folder: &Path,
        prefix: &str,
        suffix: &str,
    ) -> io::Result<Self> {
        let (file, path) = unused_name(folder, prefix, suffix, create_file)?;
        Ok(Self {
            file,
            path,
            kept: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn persist(mut self, path: &Path) -> io::Result<()> {
        fs::rename(&self.path, path)?;
        self.kept = true;
        Ok(())
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if !self.kept {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl Write for TemporaryFile {
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }

    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.file.write(data)
    }
}
