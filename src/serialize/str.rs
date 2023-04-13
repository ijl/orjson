// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::cell::RefCell;
use std::rc::Rc;
use crate::serialize::error::*;
use crate::str::*;

use serde::ser::{Serialize, Serializer};
use crate::ffi::SuspendGIL;

pub struct StrSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl StrSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: Rc<RefCell<SuspendGIL>>) -> Self {
        StrSerializer { ptr: ptr, gil: gil }
    }
}

impl Serialize for StrSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str(self.ptr, Some(Rc::clone(&self.gil)));
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}

pub struct StrSubclassSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl StrSubclassSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: Rc<RefCell<SuspendGIL>>) -> Self {
        StrSubclassSerializer { ptr: ptr, gil: gil }
    }
}

impl Serialize for StrSubclassSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uni = unicode_to_str_via_ffi(self.ptr, Some(Rc::clone(&self.gil)));
        if unlikely!(uni.is_none()) {
            err!(SerializeError::InvalidStr)
        }
        serializer.serialize_str(uni.unwrap())
    }
}
