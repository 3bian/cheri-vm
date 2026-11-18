// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::unused_name;
use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

pub struct TemporaryLink {
    pub(super) kept: bool,
    pub(super) path: PathBuf,
}

impl TemporaryLink {
    pub fn new_in(
        folder: &Path,
        prefix: &str,
        target: &Path,
    ) -> io::Result<Self> {
        let ((), path) = unused_name(folder, prefix, "", |path| {
            std::os::unix::fs::symlink(target, path)
        })?;
        Ok(Self { path, kept: false })
    }

    pub fn persist(mut self, path: &Path) -> io::Result<()> {
        fs::rename(&self.path, path)?;
        self.kept = true;
        Ok(())
    }
}

impl Drop for TemporaryLink {
    fn drop(&mut self) {
        if !self.kept {
            let _ = fs::remove_file(&self.path);
        }
    }
}
