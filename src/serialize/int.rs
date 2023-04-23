// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::GIL;
use crate::serialize::error::*;
use serde::ser::{Serialize, Serializer};
use std::ffi::c_int;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub struct IntSerializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a GIL,
}

impl<'a> IntSerializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a GIL) -> Self {
        IntSerializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for IntSerializer<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialize_i64: Option<i64>;
        if self.gil.is_released() {
            // PyLong_AsLongLongAndOverflow doesnt require GIL on errors
            let mut overflow: c_int = 0;
            let val = ffi!(PyLong_AsLongLongAndOverflow(self.ptr, &mut overflow));
            if unlikely!(overflow != 0) {
                serialize_i64 = None
            } else {
                serialize_i64 = Some(val);
            }
        } else {
            let val = ffi!(PyLong_AsLongLong(self.ptr));
            if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null()) {
                ffi!(PyErr_Clear());
                serialize_i64 = None;
            } else {
                serialize_i64 = Some(val);
            }
        }

        if let Some(val) = serialize_i64 {
            serializer.serialize_i64(val)
        } else {
            let uval: u64;
            {
                let _guard = self.gil.gil_locked();
                uval = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
                if unlikely!(uval == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
                    err!(SerializeError::Integer64Bits);
                }
            }
            serializer.serialize_u64(uval)
        }
    }
}

pub struct Int53Serializer<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a GIL,
}

impl<'a> Int53Serializer<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a GIL) -> Self {
        Int53Serializer { ptr: ptr, gil: gil }
    }
}

impl<'a> Serialize for Int53Serializer<'a> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val: i64;
        if self.gil.is_released() {
            // PyLong_AsLongLongAndOverflow doesnt require GIL on errors
            let mut overflow: c_int = 0;
            val = ffi!(PyLong_AsLongLongAndOverflow(self.ptr, &mut overflow));
            if unlikely!(overflow != 0) {
                err!(SerializeError::Integer53Bits)
            }
        } else {
            val = ffi!(PyLong_AsLongLong(self.ptr));
            if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null()) {
                err!(SerializeError::Integer53Bits)
            }
        }

        if !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&val) {
            err!(SerializeError::Integer53Bits)
        } else {
            serializer.serialize_i64(val)
        }
    }
}
