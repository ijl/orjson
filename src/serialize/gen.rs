// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct GenSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl GenSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        GenSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl<'p> Serialize for GenSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::PyIter_Next;
        let mut seq = serializer.serialize_seq(None).unwrap();

        loop {
            let obj = unsafe { PyIter_Next(self.ptr).as_mut() };
            match obj {
                Some(obj) => {
                    let value = PyObjectSerializer::new(
                        obj as *mut pyo3::ffi::PyObject,
                        self.opts,
                        self.default_calls,
                        self.recursion + 1,
                        self.default,
                    );
                    seq.serialize_element(&value)?;
                }
                None => {
                    break;
                }
            }
        }

        seq.end()
    }
}
