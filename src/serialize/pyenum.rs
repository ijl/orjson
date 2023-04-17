// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::GIL;
use crate::opt::*;
use crate::serialize::serializer::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use std::ptr::NonNull;

pub struct EnumSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: &'a GIL,
}

impl<'a> EnumSerializer<'a> {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: &'a GIL,
    ) -> Self {
        EnumSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
            gil: gil,
        }
    }
}

impl<'a> Serialize for EnumSerializer<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value: *mut pyo3_ffi::PyObject;
        {
            let mut _guard = self.gil.gil_locked();
            value = ffi!(PyObject_GetAttr(self.ptr, VALUE_STR));
            ffi!(Py_DECREF(value));
        };
        PyObjectSerializer::new(
            value,
            self.opts,
            self.default_calls,
            self.recursion,
            self.default,
            self.gil,
        )
        .serialize(serializer)
    }
}
