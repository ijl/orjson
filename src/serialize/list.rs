// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct ListSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl ListSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        ListSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl<'p> Serialize for ListSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        let slice: &[*mut pyo3_ffi::PyObject] = unsafe {
            std::slice::from_raw_parts(
                (*(self.ptr as *mut pyo3_ffi::PyListObject)).ob_item,
                ffi!(PyList_GET_SIZE(self.ptr)) as usize,
            )
        };
        for &each in slice {
            let value = PyObjectSerializer::new(
                each,
                self.opts,
                self.default_calls,
                self.recursion + 1,
                self.default,
            );
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}
