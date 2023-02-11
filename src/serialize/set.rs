// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::SerializeError;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::{null_mut, NonNull};

pub struct SetSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl SetSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        SetSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for SetSerializer {
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
        while ffi!(PyIter_Check(iter_ptr)) != 0 {
            let elem = ffi!(PyIter_Next(iter_ptr));
            if elem == null_mut() {
                if ffi!(PyErr_Occurred()).is_null() {
                    break;
                } else {
                    ffi!(Py_DECREF(iter_ptr));
                    err!(SerializeError::SetIterError)
                }
            }
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
        seq.end()
    }
}
