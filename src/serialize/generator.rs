// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::SerializeError;
use crate::serialize::serializer::*;

use crate::util::iter_next;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct GeneratorSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl GeneratorSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        GeneratorSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for GeneratorSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        while let Some(elem) = iter_next(self.ptr) {
            let value = PyObjectSerializer::new(
                elem.as_ptr(),
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            );
            seq.serialize_element(&value)?;
            ffi!(Py_DECREF(elem));
        }
        let err = ffi!(PyErr_Occurred());
        if unlikely!(!err.is_null()) {
            err!(SerializeError::GeneratorError)
        }
        seq.end()
    }
}

#[inline(always)]
pub fn is_generator(obj: *mut pyo3_ffi::PyObject) -> bool {
    ffi!(PyGen_Check(obj)) != 0
}
