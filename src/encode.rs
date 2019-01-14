// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::typeref::*;
use pyo3::prelude::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;

pub fn serialize(
    py: Python,
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
) -> PyResult<PyObject> {
    let buf: Vec<u8> = serde_json::to_vec(&SerializePyObject {
        ptr: ptr,
        default: default,
        recursion: 0,
    })
    .map_err(|error| JSONEncodeError::py_err(error.to_string()))?;
    let slice = buf.as_slice();
    Ok(unsafe {
        PyObject::from_owned_ptr(
            py,
            pyo3::ffi::PyBytes_FromStringAndSize(
                slice.as_ptr() as *const c_char,
                slice.len() as pyo3::ffi::Py_ssize_t,
            ),
        )
    })
}

struct SerializePyObject {
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    recursion: u8,
}

impl<'p> Serialize for SerializePyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj_ptr = unsafe { (*self.ptr).ob_type };
        if unsafe { obj_ptr == STR_PTR } {
            let mut str_size: pyo3::ffi::Py_ssize_t = unsafe { std::mem::uninitialized() };
            let uni =
                unsafe { pyo3::ffi::PyUnicode_AsUTF8AndSize(self.ptr, &mut str_size) as *const u8 };
            if unsafe { std::intrinsics::unlikely(uni.is_null()) } {
                return Err(ser::Error::custom(INVALID_STR));
            }
            serializer.serialize_str(unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(uni, str_size as usize))
            })
        } else if unsafe { obj_ptr == FLOAT_PTR } {
            serializer.serialize_f64(unsafe { pyo3::ffi::PyFloat_AsDouble(self.ptr) })
        } else if unsafe { obj_ptr == INT_PTR } {
            let val = unsafe { pyo3::ffi::PyLong_AsLong(self.ptr) };
            if unsafe {
                std::intrinsics::unlikely(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null())
            } {
                return Err(ser::Error::custom("Integer exceeds 64-bit max"));
            }
            serializer.serialize_i64(val)
        } else if unsafe { obj_ptr == BOOL_PTR } {
            serializer.serialize_bool(unsafe { self.ptr == TRUE })
        } else if unsafe { obj_ptr == NONE_PTR } {
            serializer.serialize_unit()
        } else if unsafe { obj_ptr == DICT_PTR } {
            let len = unsafe { pyo3::ffi::PyDict_Size(self.ptr) as usize };
            if len != 0 {
                let mut map = serializer.serialize_map(Some(len))?;
                let mut pos = 0isize;
                let mut str_size: pyo3::ffi::Py_ssize_t = unsafe { std::mem::uninitialized() };
                let mut key: *mut pyo3::ffi::PyObject = unsafe { std::mem::uninitialized() };
                let mut value: *mut pyo3::ffi::PyObject = unsafe { std::mem::uninitialized() };
                while unsafe {
                    pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0
                } {
                    if unsafe { std::intrinsics::unlikely((*key).ob_type != STR_PTR) } {
                        return Err(ser::Error::custom("Dict key must be str"));
                    }
                    let data = unsafe {
                        pyo3::ffi::PyUnicode_AsUTF8AndSize(key, &mut str_size) as *const u8
                    };
                    if unsafe { std::intrinsics::unlikely(data.is_null()) } {
                        return Err(ser::Error::custom(INVALID_STR));
                    }
                    map.serialize_entry(
                        unsafe {
                            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                                data,
                                str_size as usize,
                            ))
                        },
                        &SerializePyObject {
                            ptr: value,
                            default: self.default,
                            recursion: self.recursion,
                        },
                    )?;
                }
                map.end()
            } else {
                serializer.serialize_map(None).unwrap().end()
            }
        } else if unsafe { obj_ptr == LIST_PTR } {
            let len = unsafe { pyo3::ffi::PyList_GET_SIZE(self.ptr) as usize };
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                let mut i = 0;
                while i < len {
                    let elem =
                        unsafe { pyo3::ffi::PyList_GET_ITEM(self.ptr, i as pyo3::ffi::Py_ssize_t) };
                    i += 1;
                    seq.serialize_element(&SerializePyObject {
                        ptr: elem,
                        default: self.default,
                        recursion: self.recursion,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if unsafe { obj_ptr == TUPLE_PTR } {
            let len = unsafe { pyo3::ffi::PyTuple_GET_SIZE(self.ptr) as usize };
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                let mut i = 0;
                while i < len {
                    let elem = unsafe {
                        pyo3::ffi::PyTuple_GET_ITEM(self.ptr, i as pyo3::ffi::Py_ssize_t)
                    };
                    i += 1;
                    seq.serialize_element(&SerializePyObject {
                        ptr: elem,
                        default: self.default,
                        recursion: self.recursion,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else {
            if self.default.is_some() {
                if self.recursion > 5 {
                    return Err(ser::Error::custom(
                        "default serializer exceeds recursion limit",
                    ));
                } else {
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
                            recursion: self.recursion + 1,
                        }
                        .serialize(serializer);
                        unsafe { pyo3::ffi::Py_DECREF(default_obj) };
                        return res;
                    } else if unsafe { !pyo3::ffi::PyErr_Occurred().is_null() } {
                        return Err(ser::Error::custom(format_args!(
                            "Type raised exception in default function: {}",
                            unsafe { CStr::from_ptr((*obj_ptr).tp_name).to_string_lossy() }
                        )));
                    }
                }
            }
            Err(ser::Error::custom(format_args!(
                "Type is not JSON serializable: {}",
                unsafe { CStr::from_ptr((*obj_ptr).tp_name).to_string_lossy() }
            )))
        }
    }
}
