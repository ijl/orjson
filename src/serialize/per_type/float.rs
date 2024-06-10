// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct FloatSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl FloatSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        FloatSerializer { ptr: ptr }
    }
}

impl Serialize for FloatSerializer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr)))
    }
}
