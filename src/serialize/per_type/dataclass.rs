// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::*;
use crate::serialize::serializer::*;
use crate::str::*;
use crate::typeref::*;

use serde::ser::{Serialize, SerializeMap, Serializer};

use std::ptr::NonNull;

pub struct DataclassGenericSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DataclassGenericSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DataclassGenericSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for DataclassGenericSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            err!(SerializeError::RecursionLimit)
        }
        let dict = ffi!(PyObject_GetAttr(self.ptr, DICT_STR));
        let ob_type = ob_type!(self.ptr);
        if unlikely!(dict.is_null()) {
            ffi!(PyErr_Clear());
            DataclassFallbackSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer)
        } else if pydict_contains!(ob_type, SLOTS_STR) {
            let ret = DataclassFallbackSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer);
            ffi!(Py_DECREF(dict));
            ret
        } else {
            let ret = DataclassFastSerializer::new(
                dict,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer);
            ffi!(Py_DECREF(dict));
            ret
        }
    }
}

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
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for DataclassFastSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr));
        if unlikely!(len == 0) {
            return serializer.serialize_map(Some(0)).unwrap().end();
        }
        let mut map = serializer.serialize_map(None).unwrap();
        let mut next_key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();

        let mut pos = 0;

        ffi!(PyDict_Next(
            self.ptr,
            &mut pos,
            &mut next_key,
            &mut next_value
        ));
        for _ in 0..=ffi!(Py_SIZE(self.ptr)) as usize - 1 {
            let key = next_key;
            let value = next_value;

            ffi!(PyDict_Next(
                self.ptr,
                &mut pos,
                &mut next_key,
                &mut next_value
            ));

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
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for DataclassFallbackSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fields = ffi!(PyObject_GetAttr(self.ptr, DATACLASS_FIELDS_STR));
        debug_assert!(ffi!(Py_REFCNT(fields)) >= 2);
        ffi!(Py_DECREF(fields));
        let len = ffi!(Py_SIZE(fields)) as usize;
        if unlikely!(len == 0) {
            return serializer.serialize_map(Some(0)).unwrap().end();
        }
        let mut map = serializer.serialize_map(None).unwrap();

        let mut next_key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();

        let mut pos = 0;

        ffi!(PyDict_Next(
            fields,
            &mut pos,
            &mut next_key,
            &mut next_value
        ));
        for _ in 0..=ffi!(Py_SIZE(fields)) as usize - 1 {
            let attr = next_key;
            let field = next_value;

            ffi!(PyDict_Next(
                fields,
                &mut pos,
                &mut next_key,
                &mut next_value
            ));

            let field_type = ffi!(PyObject_GetAttr(field, FIELD_TYPE_STR));
            debug_assert!(ffi!(Py_REFCNT(field_type)) >= 2);
            ffi!(Py_DECREF(field_type));
            if unsafe { field_type as *mut pyo3_ffi::PyTypeObject != FIELD_TYPE } {
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
            debug_assert!(ffi!(Py_REFCNT(value)) >= 2);
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
