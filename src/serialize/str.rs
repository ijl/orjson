// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct StrSubclassSerializer {
    ptr: *mut pyo3::ffi::PyObject,
}

impl StrSubclassSerializer {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        StrSubclassSerializer { ptr: ptr }
    }
}

impl<'p> Serialize for StrSubclassSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let uni = ffi!(PyUnicode_AsUTF8AndSize(self.ptr, &mut str_size)) as *const u8;
        if unlikely!(uni.is_null()) {
            err!(INVALID_STR)
        }
        serializer.serialize_str(str_from_slice!(uni, str_size))
    }
}
