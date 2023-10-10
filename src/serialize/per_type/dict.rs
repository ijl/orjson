// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::*;
use crate::serialize::per_type::datetimelike::{DateTimeBuffer, DateTimeLike};
use crate::serialize::per_type::*;
use crate::serialize::serializer::{
    pyobject_to_obtype, ObType, PyObjectSerializer, RECURSION_LIMIT,
};
use crate::str::*;
use crate::typeref::*;
use compact_str::CompactString;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;
use std::ptr::NonNull;

pub struct DictGenericSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DictGenericSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DictGenericSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for DictGenericSerializer {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            err!(SerializeError::RecursionLimit)
        }
        if unlikely!(ffi!(Py_SIZE(self.ptr)) == 0) {
            serializer.serialize_map(Some(0)).unwrap().end()
        } else if opt_disabled!(self.opts, SORT_OR_NON_STR_KEYS) {
            Dict::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer)
        } else if opt_enabled!(self.opts, NON_STR_KEYS) {
            DictNonStrKey::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer)
        } else {
            DictSortedKey::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer)
        }
    }
}

pub struct Dict {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}
impl Dict {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        Dict {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}
impl Serialize for Dict {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
            let key_as_str = unicode_to_str(key);
            if unlikely!(key_as_str.is_none()) {
                err!(SerializeError::InvalidStr)
            }
            let pyvalue = PyObjectSerializer::new(
                value,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            );
            map.serialize_key(key_as_str.unwrap()).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

pub struct DictSortedKey {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DictSortedKey {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DictSortedKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for DictSortedKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        let mut items: SmallVec<[(&str, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);

        let mut next_key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();

        let mut pos = 0;

        ffi!(PyDict_Next(
            self.ptr,
            &mut pos,
            &mut next_key,
            &mut next_value
        ));
        for _ in 0..=len as usize - 1 {
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
            );
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

pub struct DictNonStrKey {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DictNonStrKey {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DictNonStrKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }

    #[cfg_attr(feature = "optimize", optimize(size))]
    fn pyobject_to_string(
        key: *mut pyo3_ffi::PyObject,
        opts: crate::opt::Opt,
    ) -> Result<CompactString, SerializeError> {
        match pyobject_to_obtype(key, opts) {
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
                let ival = ffi!(PyLong_AsLongLong(key));
                if unlikely!(ival == -1 && !ffi!(PyErr_Occurred()).is_null()) {
                    ffi!(PyErr_Clear());
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
                let dt = DateTime::new(key, opts);
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
                UUID::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(CompactString::from(key_as_str))
            }
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(key, VALUE_STR));
                debug_assert!(ffi!(Py_REFCNT(value)) >= 2);
                let ret = Self::pyobject_to_string(value, opts);
                ffi!(Py_DECREF(value));
                ret
            }
            ObType::Str => {
                // because of ObType::Enum
                let uni = unicode_to_str(key);
                if unlikely!(uni.is_none()) {
                    Err(SerializeError::InvalidStr)
                } else {
                    Ok(CompactString::from(uni.unwrap()))
                }
            }
            ObType::StrSubclass => {
                let uni = unicode_to_str_via_ffi(key);
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
            | ObType::Fragment
            | ObType::Unknown => Err(SerializeError::DictKeyInvalidType),
        }
    }
}

impl Serialize for DictNonStrKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        let mut items: SmallVec<[(CompactString, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        let opts = self.opts & NOT_PASSTHROUGH;

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

            if is_type!(ob_type!(key), STR_TYPE) {
                let uni = unicode_to_str(key);
                if unlikely!(uni.is_none()) {
                    err!(SerializeError::InvalidStr)
                }
                items.push((CompactString::from(uni.unwrap()), value));
            } else {
                match Self::pyobject_to_string(key, opts) {
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
            );
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}
