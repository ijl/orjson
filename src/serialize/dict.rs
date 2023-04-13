// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::PyDictIter;
use crate::ffi::ReleasedGIL;
use crate::opt::*;
use crate::serialize::datetime::*;
use crate::serialize::datetimelike::*;
use crate::serialize::error::*;
use crate::serialize::serializer::pyobject_to_obtype;
use crate::serialize::serializer::*;
use crate::serialize::uuid::*;
use crate::str::*;
use crate::typeref::*;
use compact_str::CompactString;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;
use std::ffi::c_int;
use std::ptr::NonNull;

pub struct Dict<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: &'a ReleasedGIL,
}

impl<'a> Dict<'a> {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: &'a ReleasedGIL,
    ) -> Self {
        Dict {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
        }
    }
}

impl<'a> Serialize for Dict<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None).unwrap();
        for (key, value) in PyDictIter::from_pyobject(self.ptr) {
            if unlikely!(unsafe { ob_type!(key) != STR_TYPE }) {
                err!(SerializeError::KeyMustBeStr)
            }
            let key_as_str = unicode_to_str(key, Some(self.gil));
            if unlikely!(key_as_str.is_none()) {
                err!(SerializeError::InvalidStr)
            }
            let pyvalue = PyObjectSerializer::new(
                value,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
                self.gil,
            );
            map.serialize_key(key_as_str.unwrap()).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

pub struct DictSortedKey<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: &'a ReleasedGIL,
}

impl<'a> DictSortedKey<'a> {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: &'a ReleasedGIL,
    ) -> Self {
        DictSortedKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
        }
    }
}

impl<'a> Serialize for DictSortedKey<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        let mut items: SmallVec<[(&str, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in PyDictIter::from_pyobject(self.ptr) {
            if unlikely!(unsafe { ob_type!(key) != STR_TYPE }) {
                err!(SerializeError::KeyMustBeStr)
            }
            let data = unicode_to_str(key, Some(self.gil));
            if unlikely!(data.is_none()) {
                err!(SerializeError::InvalidStr)
            }
            items.push((data.unwrap(), value));
        }

        items.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            let pyvalue = PyObjectSerializer::new(
                *val,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
                self.gil,
            );
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

pub struct DictNonStrKey<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: &'a ReleasedGIL,
}

impl<'a> DictNonStrKey<'a> {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: &'a ReleasedGIL,
    ) -> Self {
        DictNonStrKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
        }
    }

    #[cfg_attr(feature = "optimize", optimize(size))]
    fn pyobject_to_string(
        key: *mut pyo3_ffi::PyObject,
        opts: crate::opt::Opt,
        gil: &ReleasedGIL,
    ) -> Result<CompactString, SerializeError> {
        match pyobject_to_obtype(key, opts, gil) {
            ObType::None => Ok(CompactString::from("null")),
            ObType::Bool => {
                let key_as_str = if unsafe { key == TRUE } {
                    "true"
                } else {
                    "false"
                };
                Ok(CompactString::from(key_as_str))
            }
            ObType::Int => {
                let mut overflow: c_int = 0;
                let ival = ffi!(PyLong_AsLongLongAndOverflow(key, &mut overflow));
                if unlikely!(overflow != 0) {
                    let _guard = gil.gil_locked();
                    let uval = ffi!(PyLong_AsUnsignedLongLong(key));
                    if unlikely!(uval == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
                        return Err(SerializeError::DictIntegerKey64Bit);
                    }
                    Ok(CompactString::from(itoa::Buffer::new().format(uval)))
                } else {
                    Ok(CompactString::from(itoa::Buffer::new().format(ival)))
                }
            }
            ObType::Float => {
                let val = ffi!(PyFloat_AS_DOUBLE(key));
                if !val.is_finite() {
                    Ok(CompactString::from("null"))
                } else {
                    Ok(CompactString::from(ryu::Buffer::new().format_finite(val)))
                }
            }
            ObType::Datetime => {
                let mut buf = DateTimeBuffer::new();
                let dt = DateTime::new(key, opts, gil);
                if dt.write_buf(&mut buf, opts).is_err() {
                    return Err(SerializeError::DatetimeLibraryUnsupported);
                }
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(CompactString::from(key_as_str))
            }
            ObType::Date => {
                let mut buf = DateTimeBuffer::new();
                Date::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(CompactString::from(key_as_str))
            }
            ObType::Time => {
                let mut buf = DateTimeBuffer::new();
                let time = Time::new(key, opts);
                if time.write_buf(&mut buf).is_err() {
                    return Err(SerializeError::TimeHasTzinfo);
                }
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(CompactString::from(key_as_str))
            }
            ObType::Uuid => {
                let mut buf = arrayvec::ArrayVec::<u8, 36>::new();
                UUID::new(key, gil).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(CompactString::from(key_as_str))
            }
            ObType::Enum => {
                let value = {
                    let _guard = gil.gil_locked();
                    let val = ffi!(PyObject_GetAttr(key, VALUE_STR));
                    ffi!(Py_DECREF(val));
                    val
                };
                Self::pyobject_to_string(value, opts, gil)
            }
            ObType::Str => {
                // because of ObType::Enum
                let uni = unicode_to_str(key, Some(gil));
                if unlikely!(uni.is_none()) {
                    Err(SerializeError::InvalidStr)
                } else {
                    Ok(CompactString::from(uni.unwrap()))
                }
            }
            ObType::StrSubclass => {
                let uni = unicode_to_str_via_ffi(key, Some(gil));
                if unlikely!(uni.is_none()) {
                    Err(SerializeError::InvalidStr)
                } else {
                    Ok(CompactString::from(uni.unwrap()))
                }
            }
            ObType::Tuple
            | ObType::NumpyScalar
            | ObType::NumpyArray
            | ObType::Dict
            | ObType::List
            | ObType::Dataclass
            | ObType::Unknown => Err(SerializeError::DictKeyInvalidType),
        }
    }
}

impl<'a> Serialize for DictNonStrKey<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        let mut items: SmallVec<[(CompactString, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        let opts = self.opts & NOT_PASSTHROUGH;
        for (key, value) in PyDictIter::from_pyobject(self.ptr) {
            if is_type!(ob_type!(key), STR_TYPE) {
                let uni = unicode_to_str(key, Some(self.gil));
                if unlikely!(uni.is_none()) {
                    err!(SerializeError::InvalidStr)
                }
                items.push((CompactString::from(uni.unwrap()), value));
            } else {
                match Self::pyobject_to_string(key, opts, self.gil) {
                    Ok(key_as_str) => items.push((key_as_str, value)),
                    Err(err) => err!(err),
                }
            }
        }

        if opt_enabled!(opts, SORT_KEYS) {
            items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            let pyvalue = PyObjectSerializer::new(
                *val,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
                self.gil,
            );
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}
