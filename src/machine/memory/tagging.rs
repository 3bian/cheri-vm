// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::cheriot_tags::CheriotTags;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Tagging {
    Cheriot,
    #[default]
    None,
    Rvy,
}

impl Tagging {
    pub(crate) const fn granule_bytes(self) -> Option<usize> {
        match self {
            Self::Cheriot => Some(CheriotTags::GRANULE_BYTES),
            Self::None => None,
            Self::Rvy => Some(16),
        }
    }
}
