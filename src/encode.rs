// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::array::*;
use crate::dataclass::*;
use crate::datetime::*;
use crate::default::*;
use crate::dict::*;
use crate::exc::*;
use crate::iter::*;
use crate::typeref::*;
use crate::unicode::*;
use crate::uuid::*;
use crate::writer::*;
use pyo3::prelude::*;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::ptr::NonNull;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub const RECURSION_LIMIT: u8 = 255;

pub const NON_STR_KEYS: u16 = 1 << 8;
pub const SERIALIZE_DATACLASS: u16 = 1 << 4;
pub const SERIALIZE_NUMPY: u16 = 1 << 7;
pub const SERIALIZE_UUID: u16 = 1 << 5;
pub const SORT_KEYS: u16 = 1 << 6;
pub const STRICT_INTEGER: u16 = 1;
pub const INDENT_2: u16 = 1 << 9;

const DATACLASS_DICT_PATH: u16 = 1 << 10;
const SORT_OR_NON_STR_KEYS: u16 = SORT_KEYS | NON_STR_KEYS;

pub fn serialize(
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: u16,
) -> PyResult<NonNull<pyo3::ffi::PyObject>> {
    let mut buf = BytesWriter::new();
    let obj = SerializePyObject::new(ptr, None, opts, 0, 0, default);
    let res;
    if likely!(opts & INDENT_2 != INDENT_2) {
        res = serde_json::to_writer(&mut buf, &obj);
    } else {
        res = serde_json::to_writer_pretty(&mut buf, &obj);
    }
    match res {
        Ok(_) => Ok(unsafe { NonNull::new_unchecked(buf.finish()) }),

        Err(err) => Err(JSONEncodeError::py_err(err.to_string())),
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum ObType {
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
    ARRAY,
}

pub fn pyobject_to_obtype(obj: *mut pyo3::ffi::PyObject, opts: u16) -> ObType {
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
        } else if opts & SERIALIZE_NUMPY == SERIALIZE_NUMPY
            && ARRAY_TYPE.is_some()
            && ob_type == ARRAY_TYPE.unwrap().as_ptr()
        {
            ObType::ARRAY
        } else {
            ObType::UNKNOWN
        }
    }
}

pub struct SerializePyObject {
    ptr: *mut pyo3::ffi::PyObject,
    obtype: ObType,
    opts: u16,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl SerializePyObject {
    #[inline]
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        obtype: Option<ObType>,
        opts: u16,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        SerializePyObject {
            ptr: ptr,
            obtype: obtype.unwrap_or_else(|| pyobject_to_obtype(ptr, opts)),
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
            ObType::TIME => match Time::new(self.ptr, self.opts) {
                Ok(val) => val.serialize(serializer),
                Err(TimeError::HasTimezone) => err!(TIME_HAS_TZINFO),
            },
            ObType::UUID => UUID::new(self.ptr).serialize(serializer),
            ObType::DICT => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let len = unsafe { PyDict_GET_SIZE(self.ptr) as usize };
                if unlikely!(len == 0) {
                    serializer.serialize_map(Some(0)).unwrap().end()
                } else if likely!(
                    self.opts & SORT_OR_NON_STR_KEYS == 0
                        || self.opts & DATACLASS_DICT_PATH == DATACLASS_DICT_PATH
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
                        if unlikely!((*key).ob_type != STR_TYPE) {
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
                            None,
                            opts,
                            self.default_calls,
                            self.recursion + 1,
                            self.default,
                        ))?;
                    }
                    map.end()
                } else if self.opts & NON_STR_KEYS == NON_STR_KEYS {
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
                        seq.serialize_element(&SerializePyObject::new(
                            elem,
                            ob_type,
                            self.opts,
                            self.default_calls,
                            self.recursion + 1,
                            self.default,
                        ))?
                    }
                    seq.end()
                }
            }
            ObType::TUPLE => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                for elem in PyTupleIterator::new(self.ptr) {
                    seq.serialize_element(&SerializePyObject::new(
                        elem.as_ptr(),
                        None,
                        self.opts,
                        self.default_calls,
                        self.recursion + 1,
                        self.default,
                    ))?
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
                    SerializePyObject::new(
                        dict,
                        Some(ObType::DICT),
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
            ObType::ARRAY => match PyArray::new(self.ptr) {
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
            ObType::UNKNOWN => DefaultSerializer::new(
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
