// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use std::ptr::NonNull;

pub struct EnumSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl EnumSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        EnumSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for EnumSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = ffi!(PyObject_GetAttr(self.ptr, VALUE_STR));
        ffi!(Py_DECREF(value));
        PyObjectSerializer::new(
            value,
            self.opts,
            self.default_calls,
            self.recursion,
            self.default,
        )
        .serialize(serializer)
    }
}
