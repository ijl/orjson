// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::cell::RefCell;
use crate::opt::*;
use crate::serialize::serializer::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use std::ptr::NonNull;
use std::rc::Rc;
use crate::ffi::SuspendGIL;

pub struct EnumSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl EnumSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: Rc<RefCell<SuspendGIL>>,
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

impl Serialize for EnumSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.gil.replace_with(|v| v.restore());
        let value = ffi!(PyObject_GetAttr(self.ptr, VALUE_STR));
        ffi!(Py_DECREF(value));
        self.gil.replace_with(|v| v.release());
        PyObjectSerializer::new(
            value,
            self.opts,
            self.default_calls,
            self.recursion,
            self.default,
            Rc::clone(&self.gil),
        )
        .serialize(serializer)
    }
}
