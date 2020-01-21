// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::*;
use crate::exc::*;
use crate::typeref::*;
use pyo3::prelude::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};
use smallvec::SmallVec;
use std::ffi::CStr;
use std::io::Write;
use std::os::raw::{c_char, c_uchar};
use std::ptr::NonNull;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

const RECURSION_LIMIT: u8 = 255;

pub const STRICT_INTEGER: u8 = 1;
pub const SERIALIZE_DATACLASS: u8 = 1 << 4;
pub const SERIALIZE_UUID: u8 = 1 << 5;

macro_rules! obj_name {
    ($obj:ident) => {
        unsafe { CStr::from_ptr((*$obj).tp_name).to_string_lossy() }
    };
}

macro_rules! err {
    ($msg:expr) => {
        return Err(ser::Error::custom($msg));
    };
}

pub fn serialize(
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: u8,
) -> PyResult<NonNull<pyo3::ffi::PyObject>> {
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    match serde_json::to_writer(
        &mut buf,
        &SerializePyObject {
            ptr,
            default,
            opts,
            default_calls: 0,
            recursion: 0,
        },
    ) {
        Ok(_) => Ok(unsafe {
            NonNull::new_unchecked(pyo3::ffi::PyBytes_FromStringAndSize(
                buf.as_ptr() as *const c_char,
                buf.len() as pyo3::ffi::Py_ssize_t,
            ))
        }),

        Err(err) => Err(JSONEncodeError::py_err(err.to_string())),
    }
}
struct SerializePyObject {
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: u8,
    default_calls: u8,
    recursion: u8,
}

impl<'p> Serialize for SerializePyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj_ptr = unsafe { (*self.ptr).ob_type };
        if is_type!(obj_ptr, STR_PTR) {
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let uni = ffi!(PyUnicode_AsUTF8AndSize(self.ptr, &mut str_size)) as *const u8;
            if unlikely!(uni.is_null()) {
                err!(INVALID_STR)
            }
            serializer.serialize_str(str_from_slice!(uni, str_size))
        } else if is_type!(obj_ptr, INT_PTR) {
            let val = ffi!(PyLong_AsLongLong(self.ptr));
            if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                err!("Integer exceeds 64-bit range")
            } else if self.opts & STRICT_INTEGER == STRICT_INTEGER
                && (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
            {
                err!("Integer exceeds 53-bit range")
            }
            serializer.serialize_i64(val)
        } else if is_type!(obj_ptr, LIST_PTR) {
            let len = ffi!(PyList_GET_SIZE(self.ptr)) as usize;
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                let mut i = 0;
                while i < len {
                    if unlikely!(self.recursion == RECURSION_LIMIT) {
                        err!("Recursion limit reached")
                    }
                    let elem = ffi!(PyList_GET_ITEM(self.ptr, i as pyo3::ffi::Py_ssize_t));
                    i += 1;
                    seq.serialize_element(&SerializePyObject {
                        ptr: elem,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion + 1,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if is_type!(obj_ptr, DICT_PTR) {
            let mut map = serializer.serialize_map(None).unwrap();
            let mut pos = 0isize;
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            while unsafe { pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0 } {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!("Recursion limit reached")
                }
                if unlikely!((*key).ob_type != STR_PTR) {
                    err!("Dict key must be str")
                }
                {
                    let data = ffi!(PyUnicode_AsUTF8AndSize(key, &mut str_size)) as *const u8;
                    if unlikely!(data.is_null()) {
                        err!(INVALID_STR)
                    }
                    map.serialize_key(str_from_slice!(data, str_size)).unwrap();
                }
                map.serialize_value(&SerializePyObject {
                    ptr: value,
                    default: self.default,
                    opts: self.opts,
                    default_calls: self.default_calls,
                    recursion: self.recursion + 1,
                })?;
            }
            map.end()
        } else if is_type!(obj_ptr, BOOL_PTR) {
            serializer.serialize_bool(unsafe { self.ptr == TRUE })
        } else if is_type!(obj_ptr, NONE_PTR) {
            serializer.serialize_unit()
        } else if is_type!(obj_ptr, FLOAT_PTR) {
            serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr)))
        } else if is_type!(obj_ptr, TUPLE_PTR) {
            let len = ffi!(PyTuple_GET_SIZE(self.ptr)) as usize;
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                let mut i = 0;
                while i < len {
                    let elem = ffi!(PyTuple_GET_ITEM(self.ptr, i as pyo3::ffi::Py_ssize_t));
                    i += 1;
                    seq.serialize_element(&SerializePyObject {
                        ptr: elem,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if is_type!(obj_ptr, DATETIME_PTR) {
            let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
            match write_datetime(self.ptr, self.opts, &mut dt) {
                    Ok(_) => serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len())),
                    Err(DatetimeError::Library) => {
                    err!("datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil")
                    }
                }
        } else if is_type!(obj_ptr, DATE_PTR) {
            let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
            write_date(self.ptr, &mut dt);
            serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len()))
        } else if is_type!(obj_ptr, TIME_PTR) {
            if unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_Time)).hastzinfo == 1 } {
                err!("datetime.time must not have tzinfo set")
            }
            let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
            write_time(self.ptr, self.opts, &mut dt);
            serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len()))
        } else if is_type!(obj_ptr, UUID_PTR) && (self.opts & SERIALIZE_UUID == SERIALIZE_UUID) {
            // In Python, `self.int` is the 128-bit integer value of the UUID;
            // we can assume this will not fail, as tested in `test_uuid.py`
            let py_int = ffi!(PyObject_GetAttr(self.ptr, INT_ATTR_STR));
            // Copied in from https://github.com/PyO3/pyo3/blob/fb17d5e82f302f09b6611ac608edd1ce37504703/src/types/num.rs#L95
            // because we don't have a `pyo3::Python` reference. However, because
            // we haven't yet Py_DECREF'd the `py_int` attribute, the reference
            // to `self.int` should be valid, and this should be safe to do.
            // We know it's a `PyLongObject` because `self.int` is a 128-bit int,
            // and know that _PyLong_AsByteArray won't error, as tested in test_uuid.py
            let buffer: [c_uchar; 16] = [0; 16];
            unsafe {
                pyo3::ffi::_PyLong_AsByteArray(
                    py_int as *mut pyo3::ffi::PyLongObject,
                    buffer.as_ptr() as *const c_uchar,
                    16,
                    1, // Return a little-endian array
                    0, // Unsigned - UUIDs can't be negative
                )
            };
            ffi!(Py_DECREF(py_int));
            let value = u128::from_le_bytes(buffer);
            let mut hexadecimal: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
            write!(hexadecimal, "{:032x}", value).unwrap();
            // Now we manually format it in canonical form: 5 groups separated
            // by hyphens, 8-4-4-4-12
            // https://en.wikipedia.org/wiki/Universally_unique_identifier#Format
            let mut formatted: SmallVec<[u8; 36]> = SmallVec::with_capacity(36);
            formatted.extend_from_slice(&hexadecimal[..8]);
            formatted.push('-' as u8);
            formatted.extend_from_slice(&hexadecimal[8..12]);
            formatted.push('-' as u8);
            formatted.extend_from_slice(&hexadecimal[12..16]);
            formatted.push('-' as u8);
            formatted.extend_from_slice(&hexadecimal[16..20]);
            formatted.push('-' as u8);
            formatted.extend_from_slice(&hexadecimal[20..]);
            serializer.serialize_str(str_from_slice!(formatted.as_ptr(), 36))
        } else {
            if self.opts & SERIALIZE_DATACLASS == SERIALIZE_DATACLASS
                && ffi!(PyObject_HasAttr(self.ptr, DATACLASS_FIELDS_STR)) == 1
            {
                let fields = ffi!(PyObject_GetAttr(self.ptr, DATACLASS_FIELDS_STR));
                ffi!(Py_DECREF(fields));
                let mut map = serializer.serialize_map(None).unwrap();
                let mut pos = 0isize;
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let mut attr: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                let mut field: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                while unsafe {
                    pyo3::ffi::PyDict_Next(fields, &mut pos, &mut attr, &mut field) != 0
                } {
                    if unlikely!(self.recursion == RECURSION_LIMIT) {
                        err!("Recursion limit reached")
                    }
                    {
                        let data = ffi!(PyUnicode_AsUTF8AndSize(attr, &mut str_size)) as *const u8;
                        if unlikely!(data.is_null()) {
                            err!(INVALID_STR);
                        }
                        map.serialize_key(str_from_slice!(data, str_size)).unwrap();
                    }

                    let value = ffi!(PyObject_GetAttr(self.ptr, attr));
                    ffi!(Py_DECREF(value));

                    map.serialize_value(&SerializePyObject {
                        ptr: value,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion + 1,
                    })?;
                }
                map.end()
            } else if self.default.is_some() {
                if unlikely!(self.default_calls == RECURSION_LIMIT) {
                    err!("default serializer exceeds recursion limit")
                }
                let default_obj = unsafe {
                    pyo3::ffi::PyObject_CallFunctionObjArgs(
                        self.default.unwrap().as_ptr(),
                        self.ptr,
                        std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                    )
                };
                if !default_obj.is_null() {
                    let res = SerializePyObject {
                        ptr: default_obj,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls + 1,
                        recursion: self.recursion,
                    }
                    .serialize(serializer);
                    ffi!(Py_DECREF(default_obj));
                    res
                } else if !ffi!(PyErr_Occurred()).is_null() {
                    err!(format_args!(
                        "Type raised exception in default function: {}",
                        obj_name!(obj_ptr)
                    ))
                } else {
                    err!(format_args!(
                        "Type is not JSON serializable: {}",
                        obj_name!(obj_ptr)
                    ))
                }
            } else {
                err!(format_args!(
                    "Type is not JSON serializable: {}",
                    obj_name!(obj_ptr)
                ))
            }
        }
    }
}
