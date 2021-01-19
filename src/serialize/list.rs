// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct ListSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    len: usize,
}

impl ListSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
        len: usize,
    ) -> Self {
        ListSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
            len: len,
        }
    }
}

impl<'p> Serialize for ListSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut type_ptr = std::ptr::null_mut();
        let mut ob_type = ObType::Str;

        let mut seq = serializer.serialize_seq(None).unwrap();
        for i in 0..=self.len - 1 {
            let elem = unsafe { *(*(self.ptr as *mut pyo3::ffi::PyListObject)).ob_item.add(i) };
            if ob_type!(elem) != type_ptr {
                type_ptr = ob_type!(elem);
                ob_type = pyobject_to_obtype(elem, self.opts);
            }
            seq.serialize_element(&PyObjectSerializer::with_obtype(
                elem,
                ob_type,
                self.opts,
                self.default_calls,
                self.recursion + 1,
                self.default,
            ))?;
        }
        seq.end()
    }
}
