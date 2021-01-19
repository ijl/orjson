// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::unicode::*;

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct StrSerializer {
    ptr: *mut pyo3::ffi::PyObject,
}

impl StrSerializer {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        StrSerializer { ptr: ptr }
    }
}

impl<'p> Serialize for StrSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let uni = read_utf8_from_str(self.ptr, &mut str_size);
        if unlikely!(uni.is_null()) {
            err!(INVALID_STR)
        }
        serializer.serialize_str(str_from_slice!(uni, str_size))
    }
}

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
