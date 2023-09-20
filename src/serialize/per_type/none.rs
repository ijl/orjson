// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

pub struct NoneSerializer;

impl NoneSerializer {
    pub fn new() -> Self {
        NoneSerializer {}
    }
}

impl Serialize for NoneSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}
