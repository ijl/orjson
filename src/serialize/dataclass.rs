// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::cell::RefCell;
use crate::opt::*;
use crate::serialize::error::*;
use crate::serialize::serializer::*;
use crate::str::*;
use crate::typeref::*;

use crate::ffi::PyDictIter;
use serde::ser::{Serialize, SerializeMap, Serializer};

use std::ptr::NonNull;
use std::rc::Rc;
use crate::ffi::SuspendGIL;

pub struct DataclassFastSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl DataclassFastSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: Rc<RefCell<SuspendGIL>>,
    ) -> Self {
        DataclassFastSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
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
            let data = unicode_to_str(key, Some(Rc::clone(&self.gil)));
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
                Rc::clone(&self.gil),
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
    gil: Rc<RefCell<SuspendGIL>>,
}

impl DataclassFallbackSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: Rc<RefCell<SuspendGIL>>,
    ) -> Self {
        DataclassFallbackSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
        }
    }
}

impl Serialize for DataclassFallbackSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.gil.replace_with(|v| v.restore());
        let fields = ffi!(PyObject_GetAttr(self.ptr, DATACLASS_FIELDS_STR));
        ffi!(Py_DECREF(fields));
        let len = ffi!(Py_SIZE(fields)) as usize;
        self.gil.replace_with(|v| v.release());
        if unlikely!(len == 0) {
            return serializer.serialize_map(Some(0)).unwrap().end();
        }
        let mut map = serializer.serialize_map(None).unwrap();
        for (attr, field) in PyDictIter::from_pyobject(fields) {
            self.gil.replace_with(|v| v.restore());
            let field_type = ffi!(PyObject_GetAttr(field, FIELD_TYPE_STR));
            ffi!(Py_DECREF(field_type));
            self.gil.replace_with(|v| v.release());

            if unsafe { field_type as *mut pyo3_ffi::PyTypeObject != FIELD_TYPE } {
                continue;
            }
            let data = unicode_to_str(attr, Some(Rc::clone(&self.gil)));
            if unlikely!(data.is_none()) {
                err!(SerializeError::InvalidStr);
            }
            let key_as_str = data.unwrap();
            if key_as_str.as_bytes()[0] == b'_' {
                continue;
            }

            self.gil.replace_with(|v| v.restore());
            let value = ffi!(PyObject_GetAttr(self.ptr, attr));
            ffi!(Py_DECREF(value));
            self.gil.replace_with(|v| v.release());

            let pyvalue = PyObjectSerializer::new(
                value,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
                Rc::clone(&self.gil),
            );

            map.serialize_key(key_as_str).unwrap();
            map.serialize_value(&pyvalue)?
        }
        map.end()
    }
}
