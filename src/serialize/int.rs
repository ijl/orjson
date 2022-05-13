// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use serde::ser::{Serialize, Serializer};
use std::ffi::CStr;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub struct IntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    arbitrary_size: bool,
}

impl IntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, arbitrary_size: bool) -> Self {
        IntSerializer { ptr, arbitrary_size }
    }
}

impl<'p> Serialize for IntSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = ffi!(PyLong_AsLongLong(self.ptr));
        if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null()) {
            UIntSerializer::new(self.ptr, self.arbitrary_size).serialize(serializer)
        } else {
            serializer.serialize_i64(val)
        }
    }
}

pub struct UIntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    arbitrary_size: bool,
}

impl UIntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, arbitrary_size: bool) -> Self {
        UIntSerializer { ptr, arbitrary_size  }
    }
}

impl<'p> Serialize for UIntSerializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ffi!(PyErr_Clear());
        let val = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
        if unlikely!(val == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
            if self.arbitrary_size {
                BigIntSerializer::new(self.ptr).serialize(serializer)
            } else {
                err!(SerializeError::Integer64Bits)
            }
        } else {
            serializer.serialize_u64(val)
        }
    }
}

#[repr(transparent)]
pub struct BigIntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl BigIntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        BigIntSerializer { ptr: ptr }
    }
}

impl<'p> Serialize for BigIntSerializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ffi!(PyErr_Clear());
        let int_as_string = ffi!(PyObject_Repr(self.ptr));
        let c_buf = ffi!(PyUnicode_AsUTF8(int_as_string));
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        let str_slice: &str = c_str.to_str().unwrap();
        let str_buf: String = str_slice.to_owned();

        let raw = serde_json::value::RawValue::from_string(str_buf).unwrap();
        raw.serialize(serializer)
    }
}

#[repr(transparent)]
pub struct Int53Serializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl Int53Serializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        Int53Serializer { ptr: ptr }
    }
}

impl<'p> Serialize for Int53Serializer {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = ffi!(PyLong_AsLongLong(self.ptr));
        if unlikely!(val == -1 && !ffi!(PyErr_Occurred()).is_null())
            || (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
        {
            err!(SerializeError::Integer53Bits)
        }
        serializer.serialize_i64(val)
    }
}
