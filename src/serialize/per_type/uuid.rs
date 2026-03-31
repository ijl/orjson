// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2018-2026)

use crate::ffi::PyUuidRef;
use crate::serialize::uuid::write_uuid;
use crate::serialize::writer::SmallFixedBuffer;
use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub(crate) struct UUID {
    ob: PyUuidRef,
}

impl UUID {
    pub fn new(ptr: PyUuidRef) -> Self {
        UUID { ob: ptr }
    }
}

impl Serialize for UUID {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        write_uuid(self.ob.clone(), &mut buf);
        serializer.serialize_unit_struct(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
