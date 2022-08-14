// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::*;
use crate::serialize::serializer::*;
use crate::typeref::*;
use crate::unicode::*;

use crate::ffi::PyDictIter;
use serde::ser::{Serialize, SerializeMap, Serializer};

use std::ptr::NonNull;

pub struct DataclassFastSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DataclassFastSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DataclassFastSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for DataclassFastSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr));
        if unlikely!(len == 0) {
            return serializer.serialize_map(Some(0)).unwrap().end();
        }
        let mut map = serializer.serialize_map(None).unwrap();
        for (key, value) in PyDictIter::from_pyobject(self.ptr) {
            if unlikely!(unsafe { ob_type!(key) != STR_TYPE }) {
                err!(SerializeError::KeyMustBeStr)
            }
            let data = unicode_to_str(key);
            if unlikely!(data.is_none()) {
                err!(SerializeError::InvalidStr)
            }
            let key_as_str = data.unwrap();
            if unlikely!(key_as_str.as_bytes()[0] == b'_') {
                continue;
            }
            let pyvalue = PyObjectSerializer::new(
                value,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            );
            map.serialize_key(key_as_str).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

pub struct DataclassFallbackSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DataclassFallbackSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DataclassFallbackSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for DataclassFallbackSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fields = ffi!(PyObject_GetAttr(self.ptr, DATACLASS_FIELDS_STR));
        ffi!(Py_DECREF(fields));
        let len = ffi!(Py_SIZE(fields)) as usize;
        if unlikely!(len == 0) {
            return serializer.serialize_map(Some(0)).unwrap().end();
        }
        let mut map = serializer.serialize_map(None).unwrap();
        for (attr, field) in PyDictIter::from_pyobject(fields) {
            let field_type = ffi!(PyObject_GetAttr(field, FIELD_TYPE_STR));
            ffi!(Py_DECREF(field_type));
            if unsafe { field_type != FIELD_TYPE.as_ptr() } {
                continue;
            }
            let data = unicode_to_str(attr);
            if unlikely!(data.is_none()) {
                err!(SerializeError::InvalidStr);
            }
            let key_as_str = data.unwrap();
            if key_as_str.as_bytes()[0] == b'_' {
                continue;
            }

            let value = ffi!(PyObject_GetAttr(self.ptr, attr));
            ffi!(Py_DECREF(value));
            let pyvalue = PyObjectSerializer::new(
                value,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            );

            map.serialize_key(key_as_str).unwrap();
            map.serialize_value(&pyvalue)?
        }
        map.end()
    }
}
