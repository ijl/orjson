// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};
use core::{str::FromStr};
use crate::str::*;

#[repr(transparent)]
pub struct DecimalSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl DecimalSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        DecimalSerializer { ptr: ptr }
    }
}

impl Serialize for DecimalSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str(ffi!(PyObject_Str(self.ptr)));
        serializer.serialize_bytes(uni.unwrap().as_bytes())
    }
}

