// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::*;
use crate::exc::*;
use crate::iter::*;
use crate::typeref::*;
use crate::unicode::*;
use crate::uuid::*;
use pyo3::prelude::*;
use serde::ser::{Error, Serialize, SerializeMap, SerializeSeq, Serializer};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

const RECURSION_LIMIT: u8 = 255;

pub const SERIALIZE_DATACLASS: u8 = 1 << 4;
pub const SERIALIZE_UUID: u8 = 1 << 5;
pub const SORT_KEYS: u8 = 1 << 6;
pub const STRICT_INTEGER: u8 = 1;

const NOT_SORT_KEYS: u8 = 0b10111111;

macro_rules! obj_name {
    ($obj:ident) => {
        unsafe { CStr::from_ptr((*$obj).tp_name).to_string_lossy() }
    };
}

macro_rules! err {
    ($msg:expr) => {
        return Err(Error::custom($msg));
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
            obtype: None,
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

#[derive(Copy, Clone)]
#[repr(C)]
enum ObType {
    UNKNOWN = 1,
    STR,
    INT,
    LIST,
    DICT,
    BOOL,
    NONE,
    FLOAT,
    TUPLE,
    DATETIME,
    DATE,
    TIME,
    UUID,
    DATACLASS,
}

#[inline]
fn pyobject_to_obtype(obj: *mut pyo3::ffi::PyObject, opts: u8) -> ObType {
    unsafe {
        let ob_type = (*obj).ob_type;
        if ob_type == STR_TYPE {
            ObType::STR
        } else if ob_type == INT_TYPE {
            ObType::INT
        } else if ob_type == LIST_TYPE {
            ObType::LIST
        } else if ob_type == DICT_TYPE {
            ObType::DICT
        } else if ob_type == BOOL_TYPE {
            ObType::BOOL
        } else if ob_type == NONE_TYPE {
            ObType::NONE
        } else if ob_type == FLOAT_TYPE {
            ObType::FLOAT
        } else if ob_type == TUPLE_TYPE {
            ObType::TUPLE
        } else if ob_type == DATETIME_TYPE {
            ObType::DATETIME
        } else if ob_type == DATE_TYPE {
            ObType::DATE
        } else if ob_type == TIME_TYPE {
            ObType::TIME
        } else if ob_type == UUID_TYPE && opts & SERIALIZE_UUID == SERIALIZE_UUID {
            ObType::UUID
        } else if opts & SERIALIZE_DATACLASS == SERIALIZE_DATACLASS
            && ffi!(PyObject_HasAttr(obj, DATACLASS_FIELDS_STR)) == 1
        {
            ObType::DATACLASS
        } else {
            ObType::UNKNOWN
        }
    }
}

struct DictSortedKey {
    ptr: *mut pyo3::ffi::PyObject,
    opts: u8,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl DictSortedKey {
    #[inline(never)]
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: u8,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
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

impl<'p> Serialize for DictSortedKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(PyDict_Size(self.ptr)) as usize;
        if len == 0 {
            serializer.serialize_map(Some(0)).unwrap().end()
        } else {
            let mut items = Vec::with_capacity(len);
            let mut pos = 0isize;
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            while unsafe { pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0 } {
                if unlikely!((*key).ob_type != STR_TYPE) {
                    err!("Dict key must be str")
                }
                let data = ffi!(PyUnicode_AsUTF8AndSize(key, &mut str_size)) as *const u8;
                if unlikely!(data.is_null()) {
                    err!(INVALID_STR)
                }
                items.push((str_from_slice!(data, str_size), value));
            }

            items.sort_unstable_by(|a, b| a.0.partial_cmp(b.0).unwrap());

            let mut map = serializer.serialize_map(None).unwrap();
            for (key, val) in items.iter() {
                map.serialize_entry(
                    key,
                    &SerializePyObject {
                        ptr: *val,
                        obtype: None,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion + 1,
                    },
                )?;
            }
            map.end()
        }
    }
}

struct SerializePyObject {
    ptr: *mut pyo3::ffi::PyObject,
    obtype: Option<ObType>,
    opts: u8,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl<'p> Serialize for SerializePyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self
            .obtype
            .unwrap_or_else(|| pyobject_to_obtype(self.ptr, self.opts))
        {
            ObType::STR => {
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = read_utf8_from_str(self.ptr, &mut str_size);
                if unlikely!(uni.is_null()) {
                    err!(INVALID_STR)
                }
                serializer.serialize_str(str_from_slice!(uni, str_size))
            }
            ObType::INT => {
                let val = ffi!(PyLong_AsLongLong(self.ptr));
                if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                    err!("Integer exceeds 64-bit range")
                } else if self.opts & STRICT_INTEGER == STRICT_INTEGER
                    && (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
                {
                    err!("Integer exceeds 53-bit range")
                }
                serializer.serialize_i64(val)
            }
            ObType::NONE => serializer.serialize_unit(),
            ObType::FLOAT => serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr))),
            ObType::BOOL => serializer.serialize_bool(unsafe { self.ptr == TRUE }),
            ObType::DATETIME => DateTime::new(self.ptr, self.opts).serialize(serializer),
            ObType::DATE => Date::new(self.ptr).serialize(serializer),
            ObType::TIME => Time::new(self.ptr, self.opts).serialize(serializer),
            ObType::UUID => UUID::new(self.ptr).serialize(serializer),
            ObType::DICT => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                if unlikely!(self.opts & SORT_KEYS == SORT_KEYS) {
                    DictSortedKey::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                } else {
                    let mut map = serializer.serialize_map(None).unwrap();
                    let mut pos = 0isize;
                    let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                    let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    while unsafe {
                        pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0
                    } {
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
                            obtype: None,
                            default: self.default,
                            opts: self.opts,
                            default_calls: self.default_calls,
                            recursion: self.recursion + 1,
                        })?;
                    }
                    map.end()
                }
            }
            ObType::LIST => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let len = ffi!(PyList_GET_SIZE(self.ptr)) as usize;
                if len == 0 {
                    serializer.serialize_seq(Some(0)).unwrap().end()
                } else {
                    let slice: &[*mut pyo3::ffi::PyObject] = unsafe {
                        std::slice::from_raw_parts(
                            (*(self.ptr as *mut pyo3::ffi::PyListObject)).ob_item,
                            len,
                        )
                    };

                    let mut type_ptr = unsafe { (*(*(slice.get_unchecked(0)))).ob_type };
                    let mut ob_type = Some(pyobject_to_obtype(
                        unsafe { *(slice.get_unchecked(0)) },
                        self.opts,
                    ));

                    let mut seq = serializer.serialize_seq(None).unwrap();
                    for &elem in slice {
                        if unsafe { (*(elem)).ob_type } != type_ptr {
                            type_ptr = unsafe { (*(elem)).ob_type };
                            ob_type = Some(pyobject_to_obtype(elem, self.opts))
                        }
                        seq.serialize_element(&SerializePyObject {
                            ptr: elem,
                            obtype: ob_type,
                            default: self.default,
                            opts: self.opts,
                            default_calls: self.default_calls,
                            recursion: self.recursion + 1,
                        })?
                    }
                    seq.end()
                }
            }
            ObType::TUPLE => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                for elem in PyTupleIterator::new(self.ptr) {
                    seq.serialize_element(&SerializePyObject {
                        ptr: elem.as_ptr(),
                        obtype: None,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion + 1,
                    })?
                }
                seq.end()
            }
            ObType::DATACLASS => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let dict = ffi!(PyObject_GetAttr(self.ptr, DICT_STR));
                if !dict.is_null() {
                    ffi!(Py_DECREF(dict));
                    SerializePyObject {
                        ptr: dict,
                        obtype: Some(ObType::DICT),
                        default: self.default,
                        opts: self.opts & NOT_SORT_KEYS,
                        default_calls: self.default_calls,
                        recursion: self.recursion,
                    }
                    .serialize(serializer)
                } else {
                    unsafe { pyo3::ffi::PyErr_Clear() };
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
                            obtype: None,
                            default: self.default,
                            opts: self.opts,
                            default_calls: self.default_calls,
                            recursion: self.recursion + 1,
                        })?;
                    }
                    map.end()
                }
            }
            ObType::UNKNOWN => match self.default {
                Some(callable) => {
                    if unlikely!(self.default_calls == RECURSION_LIMIT) {
                        err!("default serializer exceeds recursion limit")
                    }
                    let obj_ptr = unsafe { (*self.ptr).ob_type };
                    let default_obj = unsafe {
                        pyo3::ffi::PyObject_CallFunctionObjArgs(
                            callable.as_ptr(),
                            self.ptr,
                            std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                        )
                    };
                    if default_obj.is_null() {
                        err!(format_args!(
                            "Type is not JSON serializable: {}",
                            obj_name!(obj_ptr)
                        ))
                    } else if !ffi!(PyErr_Occurred()).is_null() {
                        err!(format_args!(
                            "Type raised exception in default function: {}",
                            obj_name!(obj_ptr)
                        ))
                    } else {
                        let res = SerializePyObject {
                            ptr: default_obj,
                            obtype: None,
                            default: self.default,
                            opts: self.opts,
                            default_calls: self.default_calls + 1,
                            recursion: self.recursion,
                        }
                        .serialize(serializer);
                        ffi!(Py_DECREF(default_obj));
                        res
                    }
                }
                None => {
                    let obj_ptr = unsafe { (*self.ptr).ob_type };
                    err!(format_args!(
                        "Type is not JSON serializable: {}",
                        obj_name!(obj_ptr)
                    ))
                }
            },
        }
    }
}
