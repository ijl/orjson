// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::PyListIter;
use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;
use crate::ffi::ReleasedGIL;

pub struct ListSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: &'a ReleasedGIL,
}

impl<'a> ListSerializer<'a> {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: &'a ReleasedGIL,
    ) -> Self {
        ListSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
        }
    }
}

impl<'a> Serialize for ListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if ffi!(Py_SIZE(self.ptr)) == 0 {
            serializer.serialize_seq(Some(0)).unwrap().end()
        } else {
            let mut seq = serializer.serialize_seq(None).unwrap();
            for elem in PyListIter::from_pyobject(self.ptr) {
                let value = PyObjectSerializer::new(
                    elem.as_ptr(),
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                    self.gil,
                );
                seq.serialize_element(&value)?;
            }
            seq.end()
        }
    }
}
