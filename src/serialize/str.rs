// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::serialize::error::*;
use crate::unicode::*;

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct StrSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl StrSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        StrSerializer { ptr: ptr }
    }
}

impl Serialize for StrSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str(self.ptr);
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}

#[repr(transparent)]
pub struct StrSubclassSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl StrSubclassSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        StrSubclassSerializer { ptr: ptr }
    }
}

impl Serialize for StrSubclassSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str_via_ffi(self.ptr);
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}
