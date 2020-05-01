// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::*;
use crate::encode::pyobject_to_obtype;
use crate::encode::*;
use crate::exc::*;
use crate::opt::*;
use crate::typeref::*;
use crate::unicode::*;
use crate::uuid::*;
use inlinable_string::InlinableString;
use pyo3::ffi::*;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;
use std::ptr::NonNull;

#[repr(C)]
pub struct PyDictObject {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ma_used: Py_ssize_t,
    pub ma_version_tag: u64,
    pub ma_keys: *mut pyo3::ffi::PyObject,
    pub ma_values: *mut *mut pyo3::ffi::PyObject,
}

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyDict_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    (*op.cast::<PyDictObject>()).ma_used
}

pub struct DictSortedKey {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    len: usize,
}

impl DictSortedKey {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
        len: usize,
    ) -> Self {
        DictSortedKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
            len: len,
        }
    }
}

impl<'p> Serialize for DictSortedKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut items: SmallVec<[(&str, *mut pyo3::ffi::PyObject); 8]> =
            SmallVec::with_capacity(self.len);
        let mut pos = 0isize;
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
        for _ in 0..=self.len - 1 {
            unsafe {
                pyo3::ffi::_PyDict_Next(
                    self.ptr,
                    &mut pos,
                    &mut key,
                    &mut value,
                    std::ptr::null_mut(),
                )
            };
            if unlikely!(ob_type!(key) != STR_TYPE) {
                err!("Dict key must be str")
            }
            let data = read_utf8_from_str(key, &mut str_size);
            if unlikely!(data.is_null()) {
                err!(INVALID_STR)
            }
            items.push((str_from_slice!(data, str_size), value));
        }

        items.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            map.serialize_entry(
                key,
                &SerializePyObject::new(
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

enum NonStrError {
    DatetimeLibraryUnsupported,
    IntegerRange,
    InvalidStr,
    TimeTzinfo,
    UnsupportedType,
}

pub struct NonStrKey {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    len: usize,
}

impl NonStrKey {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
        len: usize,
    ) -> Self {
        NonStrKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
            len: len,
        }
    }

    fn pyobject_to_string(
        &self,
        key: *mut pyo3::ffi::PyObject,
    ) -> Result<InlinableString, NonStrError> {
        match pyobject_to_obtype(key, self.opts) {
            ObType::None => Ok(InlinableString::from("null")),
            ObType::Bool => {
                let key_as_str: &str;
                if unsafe { key == TRUE } {
                    key_as_str = "true";
                } else {
                    key_as_str = "false";
                }
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Int => {
                let val = ffi!(PyLong_AsLongLong(key));
                if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                    return Err(NonStrError::IntegerRange);
                }
                Ok(InlinableString::from(itoa::Buffer::new().format(val)))
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
                let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
                let dt = DateTime::new(key, self.opts);
                if dt.write_buf(&mut buf).is_err() {
                    return Err(NonStrError::DatetimeLibraryUnsupported);
                }
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Date => {
                let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
                Date::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Time => match Time::new(key, self.opts) {
                Ok(val) => {
                    let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
                    val.write_buf(&mut buf);
                    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                    Ok(InlinableString::from(key_as_str))
                }
                Err(TimeError::HasTimezone) => Err(NonStrError::TimeTzinfo),
            },
            ObType::Uuid => {
                let mut buf: UUIDBuffer = smallvec::SmallVec::with_capacity(64);
                UUID::new(key).write_buf(&mut buf);
                let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                Ok(InlinableString::from(key_as_str))
            }
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(key, VALUE_STR));
                ffi!(Py_DECREF(value));
                self.pyobject_to_string(value)
            }
            ObType::Str => {
                // because of ObType::Enum
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = read_utf8_from_str(key, &mut str_size);
                if unlikely!(uni.is_null()) {
                    Err(NonStrError::InvalidStr)
                } else {
                    Ok(InlinableString::from(str_from_slice!(uni, str_size)))
                }
            }
            ObType::StrSubclass => {
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = ffi!(PyUnicode_AsUTF8AndSize(key, &mut str_size)) as *const u8;
                if unlikely!(uni.is_null()) {
                    Err(NonStrError::InvalidStr)
                } else {
                    Ok(InlinableString::from(str_from_slice!(uni, str_size)))
                }
            }
            ObType::Tuple
            | ObType::Array
            | ObType::Dict
            | ObType::List
            | ObType::Dataclass
            | ObType::Unknown => Err(NonStrError::UnsupportedType),
        }
    }
}

impl<'p> Serialize for NonStrKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut items: SmallVec<[(InlinableString, *mut pyo3::ffi::PyObject); 8]> =
            SmallVec::with_capacity(self.len);
        let mut pos = 0isize;
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
        for _ in 0..=self.len - 1 {
            unsafe {
                pyo3::ffi::_PyDict_Next(
                    self.ptr,
                    &mut pos,
                    &mut key,
                    &mut value,
                    std::ptr::null_mut(),
                )
            };
            if is_type!(ob_type!(key), STR_TYPE) {
                let data = read_utf8_from_str(key, &mut str_size);
                if unlikely!(data.is_null()) {
                    err!(INVALID_STR)
                }
                items.push((
                    InlinableString::from(str_from_slice!(data, str_size)),
                    value,
                ));
            } else {
                match self.pyobject_to_string(key) {
                    Ok(key_as_str) => items.push((key_as_str, value)),
                    Err(NonStrError::TimeTzinfo) => err!(TIME_HAS_TZINFO),
                    Err(NonStrError::IntegerRange) => {
                        err!("Dict integer key must be within 64-bit range")
                    }
                    Err(NonStrError::DatetimeLibraryUnsupported) => {
                        err!(DATETIME_LIBRARY_UNSUPPORTED)
                    }
                    Err(NonStrError::InvalidStr) => err!(INVALID_STR),
                    Err(NonStrError::UnsupportedType) => {
                        err!("Dict key must a type serializable with NON_STR_KEYS")
                    }
                }
            }
        }

        if self.opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            map.serialize_entry(
                str_from_slice!(key.as_ptr(), key.len()),
                &SerializePyObject::new(
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
