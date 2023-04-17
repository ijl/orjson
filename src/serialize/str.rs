// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::serialize::error::*;
use crate::str::*;

use crate::ffi::GIL;
use serde::ser::{Serialize, Serializer};

pub struct StrSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a GIL,
}

impl<'a> StrSerializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a GIL) -> Self {
        StrSerializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for StrSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str(self.ptr, Some(self.gil));
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}

pub struct StrSubclassSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a GIL,
}

impl<'a> StrSubclassSerializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a GIL) -> Self {
        StrSubclassSerializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for StrSubclassSerializer<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str_via_ffi(self.ptr, Some(self.gil));
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}
