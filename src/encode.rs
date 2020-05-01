// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::array::*;
use crate::dataclass::*;
use crate::datetime::*;
use crate::default::*;
use crate::dict::*;
use crate::exc::*;
use crate::iter::*;
use crate::opt::*;
use crate::typeref::*;
use crate::unicode::*;
use crate::uuid::*;
use crate::writer::*;
use pyo3::prelude::*;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::os::raw::c_char;
use std::ptr::NonNull;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub const RECURSION_LIMIT: u8 = 255;

pub fn serialize(
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: Opt,
) -> PyResult<NonNull<pyo3::ffi::PyObject>> {
    let mut buf = BytesWriter::new();
    let obtype = pyobject_to_obtype(ptr, opts);
    match obtype {
        ObType::List | ObType::Dict | ObType::Dataclass | ObType::Array => {
            buf.resize(1024);
        }
        _ => {}
    }
    buf.prefetch();
    let obj = SerializePyObject::with_obtype(ptr, obtype, opts, 0, 0, default);
    let res;
    if likely!(opts & INDENT_2 != INDENT_2) {
        res = serde_json::to_writer(&mut buf, &obj);
    } else {
        res = serde_json::to_writer_pretty(&mut buf, &obj);
    }
    match res {
        Ok(_) => Ok(buf.finish()),
        Err(err) => {
            ffi!(_Py_Dealloc(buf.finish().as_ptr()));
            Err(JSONEncodeError::py_err(err.to_string()))
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObType {
    Str,
    Int,
    Bool,
    None,
    Float,
    List,
    Dict,
    Datetime,
    Date,
    Time,
    Tuple,
    Uuid,
    Dataclass,
    Array,
    Enum,
    StrSubclass,
    Unknown,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LocalPyTypeObject {
    pub ob_refcnt: pyo3::ffi::Py_ssize_t,
    pub ob_type: *mut pyo3::ffi::PyTypeObject,
    pub ma_used: pyo3::ffi::Py_ssize_t,
    pub tp_name: *const c_char,
    // ...
}

#[inline]
pub fn pyobject_to_obtype(obj: *mut pyo3::ffi::PyObject, opts: Opt) -> ObType {
    unsafe {
        let ob_type = ob_type!(obj);
        if ob_type == STR_TYPE {
            ObType::Str
        } else if ob_type == INT_TYPE {
            ObType::Int
        } else if ob_type == BOOL_TYPE {
            ObType::Bool
        } else if ob_type == NONE_TYPE {
            ObType::None
        } else if ob_type == FLOAT_TYPE {
            ObType::Float
        } else if ob_type == LIST_TYPE {
            ObType::List
        } else if ob_type == DICT_TYPE {
            ObType::Dict
        } else if ob_type == DATETIME_TYPE {
            ObType::Datetime
        } else {
            pyobject_to_obtype_unlikely(obj, opts)
        }
    }
}

macro_rules! is_subclass {
    ($ob_type:expr, $flag:ident) => {
        unsafe { (((*$ob_type).tp_flags & pyo3::ffi::$flag) != 0) }
    };
}

#[inline(never)]
pub fn pyobject_to_obtype_unlikely(obj: *mut pyo3::ffi::PyObject, opts: Opt) -> ObType {
    unsafe {
        let ob_type = ob_type!(obj);
        if ob_type == DATE_TYPE {
            ObType::Date
        } else if ob_type == TIME_TYPE {
            ObType::Time
        } else if ob_type == TUPLE_TYPE {
            ObType::Tuple
        } else if ob_type == UUID_TYPE {
            ObType::Uuid
        } else if (*(ob_type as *mut LocalPyTypeObject)).ob_type == ENUM_TYPE {
            ObType::Enum
        } else if is_subclass!(ob_type, Py_TPFLAGS_UNICODE_SUBCLASS) {
            ObType::StrSubclass
        } else if is_subclass!(ob_type, Py_TPFLAGS_LONG_SUBCLASS) {
            ObType::Int
        } else if is_subclass!(ob_type, Py_TPFLAGS_LIST_SUBCLASS) {
            ObType::List
        } else if is_subclass!(ob_type, Py_TPFLAGS_DICT_SUBCLASS) {
            ObType::Dict
        } else if ffi!(PyDict_Contains((*ob_type).tp_dict, DATACLASS_FIELDS_STR)) == 1 {
            ObType::Dataclass
        } else if opts & SERIALIZE_NUMPY != 0
            && ARRAY_TYPE.is_some()
            && ob_type == ARRAY_TYPE.unwrap().as_ptr()
        {
            ObType::Array
        } else {
            ObType::Unknown
        }
    }
}

pub struct SerializePyObject {
    ptr: *mut pyo3::ffi::PyObject,
    obtype: ObType,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl SerializePyObject {
    #[inline]
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        SerializePyObject {
            ptr: ptr,
            obtype: pyobject_to_obtype(ptr, opts),
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }

    #[inline]
    pub fn with_obtype(
        ptr: *mut pyo3::ffi::PyObject,
        obtype: ObType,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        SerializePyObject {
            ptr: ptr,
            obtype: obtype,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl<'p> Serialize for SerializePyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.obtype {
            ObType::Str => {
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = read_utf8_from_str(self.ptr, &mut str_size);
                if unlikely!(uni.is_null()) {
                    err!(INVALID_STR)
                }
                serializer.serialize_str(str_from_slice!(uni, str_size))
            }
            ObType::StrSubclass => {
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = ffi!(PyUnicode_AsUTF8AndSize(self.ptr, &mut str_size)) as *const u8;
                if unlikely!(uni.is_null()) {
                    err!(INVALID_STR)
                }
                serializer.serialize_str(str_from_slice!(uni, str_size))
            }
            ObType::Int => {
                let val = ffi!(PyLong_AsLongLong(self.ptr));
                if unlikely!(val == -1) && !ffi!(PyErr_Occurred()).is_null() {
                    err!("Integer exceeds 64-bit range")
                } else if unlikely!(self.opts & STRICT_INTEGER != 0)
                    && (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
                {
                    err!("Integer exceeds 53-bit range")
                }
                serializer.serialize_i64(val)
            }
            ObType::None => serializer.serialize_unit(),
            ObType::Float => serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr))),
            ObType::Bool => serializer.serialize_bool(unsafe { self.ptr == TRUE }),
            ObType::Datetime => DateTime::new(self.ptr, self.opts).serialize(serializer),
            ObType::Date => Date::new(self.ptr).serialize(serializer),
            ObType::Time => match Time::new(self.ptr, self.opts) {
                Ok(val) => val.serialize(serializer),
                Err(TimeError::HasTimezone) => err!(TIME_HAS_TZINFO),
            },
            ObType::Uuid => UUID::new(self.ptr).serialize(serializer),
            ObType::Dict => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let len = unsafe { PyDict_GET_SIZE(self.ptr) as usize };
                if unlikely!(len == 0) {
                    serializer.serialize_map(Some(0)).unwrap().end()
                } else if likely!(
                    self.opts & SORT_OR_NON_STR_KEYS == 0 || self.opts & DATACLASS_DICT_PATH != 0
                ) {
                    let opts = self.opts & !DATACLASS_DICT_PATH;
                    let mut map = serializer.serialize_map(None).unwrap();
                    let mut pos = 0isize;
                    let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                    let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    for _ in 0..=len - 1 {
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
                            err!(KEY_MUST_BE_STR)
                        }
                        {
                            let data = read_utf8_from_str(key, &mut str_size);
                            if unlikely!(data.is_null()) {
                                err!(INVALID_STR)
                            }
                            map.serialize_key(str_from_slice!(data, str_size)).unwrap();
                        }

                        map.serialize_value(&SerializePyObject::new(
                            value,
                            opts,
                            self.default_calls,
                            self.recursion + 1,
                            self.default,
                        ))?;
                    }
                    map.end()
                } else if self.opts & NON_STR_KEYS != 0 {
                    NonStrKey::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                        len,
                    )
                    .serialize(serializer)
                } else {
                    DictSortedKey::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                        len,
                    )
                    .serialize(serializer)
                }
            }
            ObType::List => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let len = ffi!(PyList_GET_SIZE(self.ptr)) as usize;
                if len == 0 {
                    serializer.serialize_seq(Some(0)).unwrap().end()
                } else {
                    let mut type_ptr = std::ptr::null_mut();
                    let mut ob_type = ObType::Str;

                    let mut seq = serializer.serialize_seq(None).unwrap();
                    for i in 0..=len - 1 {
                        let elem = unsafe {
                            *(*(self.ptr as *mut pyo3::ffi::PyListObject))
                                .ob_item
                                .offset(i as isize)
                        };
                        if ob_type!(elem) != type_ptr {
                            type_ptr = ob_type!(elem);
                            ob_type = pyobject_to_obtype(elem, self.opts);
                        }
                        seq.serialize_element(&SerializePyObject::with_obtype(
                            elem,
                            ob_type,
                            self.opts,
                            self.default_calls,
                            self.recursion + 1,
                            self.default,
                        ))?;
                    }
                    seq.end()
                }
            }
            ObType::Tuple => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                for elem in PyTupleIterator::new(self.ptr) {
                    seq.serialize_element(&SerializePyObject::new(
                        elem.as_ptr(),
                        self.opts,
                        self.default_calls,
                        self.recursion + 1,
                        self.default,
                    ))?
                }
                seq.end()
            }
            ObType::Dataclass => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let dict = ffi!(PyObject_GetAttr(self.ptr, DICT_STR));
                if !dict.is_null() {
                    ffi!(Py_DECREF(dict));
                    SerializePyObject::with_obtype(
                        dict,
                        ObType::Dict,
                        self.opts | DATACLASS_DICT_PATH,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                } else {
                    unsafe { pyo3::ffi::PyErr_Clear() };
                    DataclassSerializer::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                }
            }
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(self.ptr, VALUE_STR));
                ffi!(Py_DECREF(value));
                SerializePyObject::new(
                    value,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(serializer)
            }
            ObType::Array => match PyArray::new(self.ptr) {
                Ok(val) => val.serialize(serializer),
                Err(PyArrayError::Malformed) => err!("numpy array is malformed"),
                Err(PyArrayError::NotContiguous) | Err(PyArrayError::UnsupportedDataType) => {
                    DefaultSerializer::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                }
            },
            ObType::Unknown => DefaultSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
        }
    }
}
