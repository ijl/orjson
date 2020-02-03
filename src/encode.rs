// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::*;
use crate::exc::*;
use crate::iter::*;
use crate::typeref::*;
use crate::unicode::*;
use crate::uuid::*;
use pyo3::prelude::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};
use std::ffi::CStr;
use std::os::raw::c_char;
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
        if is_type!(obj_ptr, STR_TYPE) {
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let uni = read_utf8_from_str(self.ptr, &mut str_size);
            if unlikely!(uni.is_null()) {
                err!(INVALID_STR)
            }
            serializer.serialize_str(str_from_slice!(uni, str_size))
        } else if is_type!(obj_ptr, INT_TYPE) {
            let val = ffi!(PyLong_AsLongLong(self.ptr));
            if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                err!("Integer exceeds 64-bit range")
            } else if self.opts & STRICT_INTEGER == STRICT_INTEGER
                && (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
            {
                err!("Integer exceeds 53-bit range")
            }
            serializer.serialize_i64(val)
        } else if is_type!(obj_ptr, LIST_TYPE) {
            if unlikely!(self.recursion == RECURSION_LIMIT) {
                err!("Recursion limit reached")
            }
            let mut seq = serializer.serialize_seq(None).unwrap();
            for elem in PyListIterator::new(self.ptr) {
                seq.serialize_element(&SerializePyObject {
                    ptr: elem.as_ptr(),
                    default: self.default,
                    opts: self.opts,
                    default_calls: self.default_calls,
                    recursion: self.recursion + 1,
                })?
            }
            seq.end()
        } else if is_type!(obj_ptr, DICT_TYPE) {
            if unlikely!(self.recursion == RECURSION_LIMIT) {
                err!("Recursion limit reached")
            }
            let mut map = serializer.serialize_map(None).unwrap();
            let mut pos = 0isize;
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            while unsafe { pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0 } {
                if unlikely!((*key).ob_type != STR_TYPE) {
                    err!("Dict key must be str")
                }
                {
                    let data = read_utf8_from_str(key, &mut str_size);
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
        } else if is_type!(obj_ptr, BOOL_TYPE) {
            serializer.serialize_bool(unsafe { self.ptr == TRUE })
        } else if is_type!(obj_ptr, NONE_TYPE) {
            serializer.serialize_unit()
        } else if is_type!(obj_ptr, FLOAT_TYPE) {
            serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr)))
        } else if is_type!(obj_ptr, TUPLE_TYPE) {
            let mut seq = serializer.serialize_seq(None).unwrap();
            for elem in PyTupleIterator::new(self.ptr) {
                seq.serialize_element(&SerializePyObject {
                    ptr: elem.as_ptr(),
                    default: self.default,
                    opts: self.opts,
                    default_calls: self.default_calls,
                    recursion: self.recursion + 1,
                })?
            }
            seq.end()
        } else if is_type!(obj_ptr, DATETIME_TYPE) {
            let mut buf: DateTimeBuffer = heapless::Vec::new();
            match write_datetime(self.ptr, self.opts, &mut buf) {
                    Ok(_) => serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len())),
                    Err(DatetimeError::Library) => {
                    err!("datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil")
                    }
                }
        } else if is_type!(obj_ptr, DATE_TYPE) {
            Date::new(self.ptr).serialize(serializer)
        } else if is_type!(obj_ptr, TIME_TYPE) {
            if unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_Time)).hastzinfo == 1 } {
                err!("datetime.time must not have tzinfo set")
            }
            Time::new(self.ptr, self.opts).serialize(serializer)
        } else if self.opts & SERIALIZE_UUID == SERIALIZE_UUID && is_type!(obj_ptr, UUID_TYPE) {
            let mut buf: UUIDBuffer = heapless::Vec::new();
            write_uuid(self.ptr, &mut buf);
            serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
        } else {
            if self.opts & SERIALIZE_DATACLASS == SERIALIZE_DATACLASS
                && ffi!(PyObject_HasAttr(self.ptr, DATACLASS_FIELDS_STR)) == 1
            {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!("Recursion limit reached")
                }
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
                    {
                        let data = read_utf8_from_str(attr, &mut str_size);
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
