// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::SerializeError;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::{null_mut, NonNull};

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
        while ffi!(PyIter_Check(self.ptr)) != 0 {
            let elem = ffi!(PyIter_Next(self.ptr));
            if elem == null_mut() {
                if unlikely!(!ffi!(PyErr_Occurred()).is_null()) {
                    err!(SerializeError::GeneratorError)
                } else {
                    break;
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
        seq.end()
    }
}

#[inline(always)]
pub fn is_generator(obj: *mut pyo3_ffi::PyObject) -> bool {
    if unlikely!(obj.is_null()) {
        false
    } else {
        ffi!(PyGen_Check(obj)) != 0
    }
}
