// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::SerializeError;
use crate::serialize::serializer::*;

use crate::util::iter_next;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct AnySetSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl AnySetSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        AnySetSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for AnySetSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let iter_ptr = ffi!(PyObject_GetIter(self.ptr));
        if unlikely!(iter_ptr.is_null()) {
            err!(SerializeError::GetIterError(nonnull!(self.ptr)))
        }
        let mut seq = serializer.serialize_seq(None).unwrap();
        while let Some(elem) = iter_next(iter_ptr) {
            let value = PyObjectSerializer::new(
                elem,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            );
            seq.serialize_element(&value)?;
            ffi!(Py_DECREF(elem));
        }
        ffi!(Py_DECREF(iter_ptr));
        let err = ffi!(PyErr_Occurred());
        if unlikely!(!err.is_null()) {
            err!(SerializeError::SetIterError)
        }
        seq.end()
    }
}

#[inline(always)]
pub fn is_any_set(obj: *mut pyo3_ffi::PyObject, passthrough_subclass: bool) -> bool {
    ffi!(PyAnySet_CheckExact(obj)) != 0 || (passthrough_subclass && ffi!(PyAnySet_Check(obj)) != 0)
}
