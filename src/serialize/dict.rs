// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::PyDict_GET_SIZE;
use crate::opt::*;
use crate::serialize::datetime::*;
use crate::serialize::datetimelike::*;
use crate::serialize::error::*;
use crate::serialize::serializer::pyobject_to_obtype;
use crate::serialize::serializer::*;
use crate::serialize::uuid::*;
use crate::typeref::*;
use crate::unicode::*;
use inlinable_string::InlinableString;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;
use std::ptr::addr_of_mut;
use std::ptr::NonNull;

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
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None).unwrap();
        let mut pos = 0isize;
        let mut key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        for _ in 0..=unsafe { PyDict_GET_SIZE(self.ptr) as usize } - 1 {
            unsafe {
                pyo3_ffi::_PyDict_Next(
                    self.ptr,
                    addr_of_mut!(pos),
                    addr_of_mut!(key),
                    addr_of_mut!(value),
                    std::ptr::null_mut(),
                )
            };
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
                self.recursion + 1,
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
        let len = unsafe { PyDict_GET_SIZE(self.ptr) as usize };
        let mut items: SmallVec<[(&str, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        let mut pos = 0isize;
        let mut key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        for _ in 0..=len - 1 {
            unsafe {
                pyo3_ffi::_PyDict_Next(
                    self.ptr,
                    addr_of_mut!(pos),
                    addr_of_mut!(key),
                    addr_of_mut!(value),
                    std::ptr::null_mut(),
                )
            };
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
            map.serialize_entry(
                key,
                &PyObjectSerializer::new(
                    *val,
                    self.opts,
                    self.default_calls,
                    self.recursion + 1,
                    self.default,
                ),
            )?;
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

    fn pyobject_to_string(
        &self,
        key: *mut pyo3_ffi::PyObject,
        opts: crate::opt::Opt,
    ) -> Result<InlinableString, SerializeError> {
        match pyobject_to_obtype(key, opts) {
            ObType::None => Ok(InlinableString::from("null")),
            ObType::Bool => {
                let key_as_str = if unsafe { key == TRUE } {
                    "true"
                } else {
                    "false"
                };
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Int => {
                let ival = ffi!(PyLong_AsLongLong(key));
                if unlikely!(ival == -1 && !ffi!(PyErr_Occurred()).is_null()) {
                    ffi!(PyErr_Clear());
                    let uval = ffi!(PyLong_AsUnsignedLongLong(key));
                    if unlikely!(uval == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
                        return Err(SerializeError::DictIntegerKey64Bit);
                    }
                    Ok(InlinableString::from(itoa::Buffer::new().format(uval)))
                } else {
                    Ok(InlinableString::from(itoa::Buffer::new().format(ival)))
                }
            }
            ObType::Float => {
                let val = ffi!(PyFloat_AS_DOUBLE(key));
                if !val.is_finite() {
                    Ok(InlinableString::from("null"))
                } else {
                    Ok(InlinableString::from(ryu::Buffer::new().format_finite(val)))
                }
            }
            ObType::Datetime => {
                let mut buf = DateTimeBuffer::new();
                let dt = DateTime::new(key, opts);
                if dt.write_buf(&mut buf, opts).is_err() {
                    return Err(SerializeError::DatetimeLibraryUnsupported);
                }
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Date => {
                let mut buf = DateTimeBuffer::new();
                Date::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Time => match Time::new(key, opts) {
                Ok(val) => {
                    let mut buf = DateTimeBuffer::new();
                    val.write_buf(&mut buf);
                    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                    Ok(InlinableString::from(key_as_str))
                }
                Err(TimeError::HasTimezone) => Err(SerializeError::TimeHasTzinfo),
            },
            ObType::Uuid => {
                let mut buf = arrayvec::ArrayVec::<u8, 36>::new();
                UUID::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(key, VALUE_STR));
                ffi!(Py_DECREF(value));
                self.pyobject_to_string(value, opts)
            }
            ObType::Str => {
                // because of ObType::Enum
                let uni = unicode_to_str(key);
                if unlikely!(uni.is_none()) {
                    Err(SerializeError::InvalidStr)
                } else {
                    Ok(InlinableString::from(uni.unwrap()))
                }
            }
            ObType::StrSubclass => {
                let uni = unicode_to_str_via_ffi(key);
                if unlikely!(uni.is_none()) {
                    Err(SerializeError::InvalidStr)
                } else {
                    Ok(InlinableString::from(uni.unwrap()))
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

impl Serialize for DictNonStrKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = unsafe { PyDict_GET_SIZE(self.ptr) as usize };
        let mut items: SmallVec<[(InlinableString, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        let mut pos = 0isize;
        let mut key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let opts = self.opts & NOT_PASSTHROUGH;
        for _ in 0..=len - 1 {
            unsafe {
                pyo3_ffi::_PyDict_Next(
                    self.ptr,
                    addr_of_mut!(pos),
                    addr_of_mut!(key),
                    addr_of_mut!(value),
                    std::ptr::null_mut(),
                )
            };
            if is_type!(ob_type!(key), STR_TYPE) {
                let data = unicode_to_str(key);
                if unlikely!(data.is_none()) {
                    err!(SerializeError::InvalidStr)
                }
                items.push((InlinableString::from(data.unwrap()), value));
            } else {
                match self.pyobject_to_string(key, opts) {
                    Ok(key_as_str) => items.push((key_as_str, value)),
                    Err(err) => err!(err),
                }
            }
        }

        if opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            map.serialize_entry(
                str_from_slice!(key.as_ptr(), key.len()),
                &PyObjectSerializer::new(
                    *val,
                    self.opts,
                    self.default_calls,
                    self.recursion + 1,
                    self.default,
                ),
            )?;
        }
        map.end()
    }
}
