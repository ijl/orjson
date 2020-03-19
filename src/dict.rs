// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::*;
use crate::encode::pyobject_to_obtype;
use crate::encode::*;
use crate::exc::*;
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
    opts: u16,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    len: usize,
}

impl DictSortedKey {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: u16,
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
            if unlikely!((*key).ob_type != STR_TYPE) {
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
                    None,
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

pub struct NonStrKey {
    ptr: *mut pyo3::ffi::PyObject,
    opts: u16,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    len: usize,
}

impl NonStrKey {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: u16,
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
            if unsafe { (*key).ob_type == STR_TYPE } {
                let data = read_utf8_from_str(key, &mut str_size);
                if unlikely!(data.is_null()) {
                    err!(INVALID_STR)
                }
                items.push((
                    InlinableString::from(str_from_slice!(data, str_size)),
                    value,
                ));
            } else {
                match pyobject_to_obtype(key, self.opts | SERIALIZE_UUID) {
                    ObType::NONE => {
                        items.push((InlinableString::from("null"), value));
                    }
                    ObType::BOOL => {
                        let key_as_str: &str;
                        if unsafe { key == TRUE } {
                            key_as_str = "true";
                        } else {
                            key_as_str = "false";
                        }
                        items.push((InlinableString::from(key_as_str), value));
                    }
                    ObType::INT => {
                        let val = ffi!(PyLong_AsLongLong(key));
                        if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                            err!("Dict integer key must be within 64-bit range")
                        }
                        items.push((
                            InlinableString::from(itoa::Buffer::new().format(val)),
                            value,
                        ));
                    }
                    ObType::FLOAT => {
                        let val = ffi!(PyFloat_AS_DOUBLE(key));
                        if !val.is_finite() {
                            items.push((InlinableString::from("null"), value));
                        } else {
                            items.push((
                                InlinableString::from(ryu::Buffer::new().format_finite(val)),
                                value,
                            ));
                        }
                    }
                    ObType::DATETIME => {
                        let mut buf: DateTimeBuffer = heapless::Vec::new();
                        let dt = DateTime::new(key, self.opts);
                        if dt.write_buf(&mut buf).is_err() {
                            err!(DATETIME_LIBRARY_UNSUPPORTED)
                        }
                        let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                        items.push((InlinableString::from(key_as_str), value));
                    }
                    ObType::DATE => {
                        let mut buf: DateTimeBuffer = heapless::Vec::new();
                        Date::new(key).write_buf(&mut buf);
                        let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                        items.push((InlinableString::from(key_as_str), value));
                    }
                    ObType::TIME => match Time::new(key, self.opts) {
                        Ok(val) => {
                            let mut buf: DateTimeBuffer = heapless::Vec::new();
                            val.write_buf(&mut buf);
                            let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                            items.push((InlinableString::from(key_as_str), value));
                        }
                        Err(TimeError::HasTimezone) => err!(TIME_HAS_TZINFO),
                    },
                    ObType::UUID => {
                        let mut buf: UUIDBuffer = heapless::Vec::new();
                        UUID::new(key).write_buf(&mut buf);
                        let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
                        items.push((InlinableString::from(key_as_str), value));
                    }
                    ObType::TUPLE
                    | ObType::ARRAY
                    | ObType::DICT
                    | ObType::LIST
                    | ObType::DATACLASS
                    | ObType::UNKNOWN => {
                        err!("Dict key must a type serializable with NON_STR_KEYS")
                    }
                    ObType::STR => unsafe { std::hint::unreachable_unchecked() },
                }
            }
        }

        if self.opts & SORT_KEYS == SORT_KEYS {
            items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            map.serialize_entry(
                str_from_slice!(key.as_ptr(), key.len()),
                &SerializePyObject::new(
                    *val,
                    None,
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
