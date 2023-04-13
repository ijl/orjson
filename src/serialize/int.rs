// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::cell::RefCell;
use std::ffi::c_int;
use std::rc::Rc;
use crate::serialize::error::*;
use serde::ser::{Serialize, Serializer};
use crate::ffi::SuspendGIL;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub struct IntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl IntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: Rc<RefCell<SuspendGIL>>) -> Self {
        IntSerializer { ptr: ptr, gil: gil }
    }
}

impl Serialize for IntSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut overflow: c_int = 0;
        let val = ffi!(PyLong_AsLongLongAndOverflow(self.ptr, &mut overflow));
        if unlikely!(overflow != 0) {
            UIntSerializer::new(self.ptr, Rc::clone(&self.gil)).serialize(serializer)
        } else {
            serializer.serialize_i64(val)
        }
    }
}

pub struct UIntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl UIntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: Rc<RefCell<SuspendGIL>>) -> Self {
        UIntSerializer { ptr: ptr, gil: gil }
    }
}

impl Serialize for UIntSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.gil.replace_with(|v| v.restore());
        ffi!(PyErr_Clear());
        let val = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
        if unlikely!(val == u64::MAX) {
            if ffi!(PyErr_Occurred()).is_null() {
                self.gil.replace_with(|v| v.release());
                serializer.serialize_u64(val)
            } else {
                self.gil.replace_with(|v| v.release());
                err!(SerializeError::Integer64Bits)
            }
        } else {
            self.gil.replace_with(|v| v.release());
            serializer.serialize_u64(val)
        }
    }
}

pub struct Int53Serializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl Int53Serializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        Int53Serializer { ptr: ptr }
    }
}

impl Serialize for Int53Serializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut overflow: c_int = 0;
        let val = ffi!(PyLong_AsLongLongAndOverflow(self.ptr, &mut overflow));
        if unlikely!(overflow != 0) {
            err!(SerializeError::Integer53Bits)
        } else if !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&val) {
            err!(SerializeError::Integer53Bits)
        } else {
            serializer.serialize_i64(val)
        }
    }
}
