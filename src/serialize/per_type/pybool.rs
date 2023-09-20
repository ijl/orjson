// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct BoolSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl BoolSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        BoolSerializer { ptr: ptr }
    }
}

impl Serialize for BoolSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(unsafe { self.ptr == crate::typeref::TRUE })
    }
}
