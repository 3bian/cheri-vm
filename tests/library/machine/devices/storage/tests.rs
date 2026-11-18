// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::devices::Device;
use crate::machine::devices::storage::Storage;
use crate::machine::devices::storage::registers::Register;
use crate::machine::devices::storage::registers::SECTOR_BYTES;

impl Storage {
    pub(crate) fn eject(&mut self) {
        self.media = None;
        self.status = 0;
    }

    pub(crate) fn from_image(
        image: &[u8],
        read_only: bool,
    ) -> Result<Self, EmulatorError> {
        if !image.len().is_multiple_of(SECTOR_BYTES) {
            return Err(EmulatorError::Configuration(
                "storage image length must be a multiple of 512 bytes".into(),
            ));
        }

        let mut storage = Self::new(image.len() / SECTOR_BYTES, read_only)?;

        if let Some(media) = &mut storage.media {
            media.copy_from_slice(image);
        }

        Ok(storage)
    }

    pub(crate) fn media_bytes(&self) -> Option<&[u8]> {
        self.media.as_deref()
    }
}

#[test]
fn a_sector_round_trips_and_read_only_or_missing_media_is_an_error() {
    let mut storage = Storage::new(1, false).unwrap();
    storage
        .write(Register::Data.offset(), 4, 0x12345678)
        .unwrap();
    storage.write(Register::Command.offset(), 4, 2).unwrap();
    storage.write(Register::Data.offset(), 4, 0).unwrap();
    storage.write(Register::Command.offset(), 4, 1).unwrap();
    assert_eq!(
        storage.read(Register::Data.offset(), 4).unwrap(),
        0x12345678
    );
    storage.eject();
    storage.write(Register::Command.offset(), 4, 1).unwrap();
    assert_eq!(storage.read(Register::Status.offset(), 4).unwrap() & 2, 2);
    let mut readonly = Storage::new(1, true).unwrap();
    readonly.write(Register::Command.offset(), 4, 2).unwrap();
    assert_eq!(readonly.read(Register::Status.offset(), 4).unwrap() & 2, 2);
}

#[test]
fn an_image_round_trips_and_errors_and_interrupts_are_acknowledged() {
    let mut image = vec![0u8; 512];
    image[508..].copy_from_slice(&0x12345678u32.to_le_bytes());
    let mut storage = Storage::from_image(&image, false).unwrap();
    storage.write(Register::Control.offset(), 4, 1).unwrap();
    storage.write(Register::Command.offset(), 4, 1).unwrap();
    storage.write(Register::WordIndex.offset(), 4, 127).unwrap();
    assert_eq!(
        storage.read(Register::Data.offset(), 4).unwrap(),
        0x12345678
    );
    storage
        .write(Register::Data.offset(), 4, 0xdeadbeef)
        .unwrap();
    storage.write(Register::Command.offset(), 4, 2).unwrap();
    assert_eq!(
        &storage.media_bytes().unwrap()[508..],
        &0xdeadbeefu32.to_le_bytes()
    );
    assert!(storage.interrupt().is_some());
    storage.write(Register::Status.offset(), 4, 3).unwrap();
    assert!(storage.interrupt().is_none());
    storage
        .write(Register::Sector.offset(), 4, u32::MAX)
        .unwrap();
    storage.write(Register::Command.offset(), 4, 2).unwrap();
    assert_eq!(storage.read(Register::Status.offset(), 4).unwrap(), 6);
    assert!(storage.write(Register::WordIndex.offset(), 4, 128).is_err());
    assert_eq!(storage.read(Register::WordIndex.offset(), 4).unwrap(), 127);
    assert!(Storage::from_image(&[], false).is_err());
    assert!(Storage::from_image(&[0; 513], false).is_err());
}
