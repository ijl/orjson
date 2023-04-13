// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::ffi::c_int;
use crate::serialize::error::*;
use serde::ser::{Serialize, Serializer};
use crate::ffi::ReleasedGIL;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub struct IntSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a ReleasedGIL,
}

impl<'a> IntSerializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a ReleasedGIL) -> Self {
        IntSerializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for IntSerializer<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut overflow: c_int = 0;
        let val = ffi!(PyLong_AsLongLongAndOverflow(self.ptr, &mut overflow));
        if unlikely!(overflow != 0) {
            UIntSerializer::new(self.ptr, self.gil).serialize(serializer)
        } else {
            serializer.serialize_i64(val)
        }
    }
}

pub struct UIntSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a ReleasedGIL,
}

impl<'a> UIntSerializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a ReleasedGIL) -> Self {
        UIntSerializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for UIntSerializer<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut _guard = self.gil.gil_locked();
        ffi!(PyErr_Clear());
        let val = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
        if unlikely!(val == u64::MAX) {
            if ffi!(PyErr_Occurred()).is_null() {
                serializer.serialize_u64(val)
            } else {
                err!(SerializeError::Integer64Bits)
            }
        } else {
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
