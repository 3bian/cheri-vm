// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{create_directory, unused_name};
use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

pub struct TemporaryDirectory {
    pub(super) path: PathBuf,
}

impl TemporaryDirectory {
    pub fn new() -> io::Result<Self> {
        let ((), path) = unused_name(
            &std::env::temp_dir(),
            "cheriot-",
            "",
            create_directory,
        )?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
