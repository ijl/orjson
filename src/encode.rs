// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::typeref::*;
use pyo3::prelude::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};
use smallvec::SmallVec;
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

const HYPHEN: u8 = 45; // "-"
const PLUS: u8 = 43; // "+"
const ZERO: u8 = 48; // "0"
const T: u8 = 84; // "T"
const COLON: u8 = 58; // ":"
const PERIOD: u8 = 46; // ":"

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
        } else {
            if unsafe { obj_ptr == TUPLE_PTR } {
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
            } else if unsafe { obj_ptr == DATETIME_PTR } {
                if unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 0 } {
                    return Err(ser::Error::custom(
                        "datetime.datetime must have tzinfo set; use datetime.timezone.utc if UTC",
                    ));
                }
                let tzinfo = unsafe { pyo3::ffi::PyDateTime_DATE_GET_TZINFO(self.ptr) };
                let offset: *mut pyo3::ffi::PyObject;
                if unsafe { pyo3::ffi::PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR) == 1 } {
                    // pendulum
                    offset = unsafe {
                        pyo3::ffi::PyObject_CallMethodObjArgs(
                            self.ptr,
                            UTCOFFSET_METHOD_STR,
                            std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                        )
                    };
                } else if unsafe { pyo3::ffi::PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR) == 1 }
                {
                    // pytz
                    offset = unsafe {
                        pyo3::ffi::PyObject_CallMethodObjArgs(
                            pyo3::ffi::PyObject_CallMethodObjArgs(
                                tzinfo,
                                NORMALIZE_METHOD_STR,
                                self.ptr,
                                std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                            ),
                            UTCOFFSET_METHOD_STR,
                            std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                        )
                    };
                } else if unsafe { pyo3::ffi::PyObject_HasAttr(tzinfo, DST_STR) == 1 } {
                    // dateutil/arrow, datetime.timezone.utc
                    offset = unsafe {
                        pyo3::ffi::PyObject_CallMethodObjArgs(
                            tzinfo,
                            UTCOFFSET_METHOD_STR,
                            self.ptr,
                            std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                        )
                    };
                } else {
                    return Err(ser::Error::custom(
                    "datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil",
                ));
                };

                let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);

                dt.extend(
                    itoa::Buffer::new()
                        .format(unsafe { pyo3::ffi::PyDateTime_GET_YEAR(self.ptr) as i32 })
                        .bytes(),
                );

                dt.push(HYPHEN);

                {
                    let month = unsafe { pyo3::ffi::PyDateTime_GET_MONTH(self.ptr) as u8 };
                    if month < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(month).bytes());

                    dt.push(HYPHEN);
                }

                {
                    let day = unsafe { pyo3::ffi::PyDateTime_GET_DAY(self.ptr) as u8 };
                    if day < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(day).bytes());

                    dt.push(T);
                }

                {
                    let hour = unsafe { pyo3::ffi::PyDateTime_DATE_GET_HOUR(self.ptr) as u8 };
                    if hour < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(hour).bytes());

                    dt.push(COLON);
                }

                {
                    let minute = unsafe { pyo3::ffi::PyDateTime_DATE_GET_MINUTE(self.ptr) as u8 };
                    if minute < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(minute).bytes());
                }

                dt.push(COLON);

                {
                    let second = unsafe { pyo3::ffi::PyDateTime_DATE_GET_SECOND(self.ptr) as u8 };
                    if second < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(second).bytes());
                }

                {
                    let microsecond =
                        unsafe { pyo3::ffi::PyDateTime_DATE_GET_MICROSECOND(self.ptr) as u32 };
                    if microsecond != 0 {
                        dt.push(PERIOD);
                        dt.extend(itoa::Buffer::new().format(microsecond).bytes());
                    }
                }
                let mut offset_second =
                    unsafe { pyo3::ffi::PyDateTime_DELTA_GET_SECONDS(offset) as i32 };
                if offset_second == 0 {
                    dt.push(PLUS);
                    dt.push(ZERO);
                    dt.push(ZERO);
                    dt.push(COLON);
                    dt.push(ZERO);
                    dt.push(ZERO);
                } else {
                    if unsafe { pyo3::ffi::PyDateTime_DELTA_GET_DAYS(offset) == -1 } {
                        // datetime.timedelta(days=-1, seconds=68400) -> -05:00
                        dt.push(HYPHEN);
                        offset_second = 86400 - offset_second
                    } else {
                        // datetime.timedelta(seconds=37800) -> +10:30
                        dt.push(PLUS);
                    }
                    {
                        let offset_minute = offset_second / 60;
                        let offset_hour = offset_minute / 60;
                        if offset_hour < 10 {
                            dt.push(ZERO);
                        }
                        dt.extend(itoa::Buffer::new().format(offset_hour).bytes());
                        dt.push(COLON);

                        let mut offset_minute_print = offset_minute % 60;

                        {
                            // https://tools.ietf.org/html/rfc3339#section-5.8
                            // "exactly 19 minutes and 32.13 seconds ahead of UTC"
                            // "closest representable UTC offset"
                            //  "+20:00"
                            let offset_excess_second =
                                offset_second - (offset_minute_print * 60 + offset_hour * 3600);
                            if offset_excess_second >= 30 {
                                offset_minute_print += 1;
                            }
                        }

                        if offset_minute_print < 10 {
                            dt.push(ZERO);
                        }
                        dt.extend(itoa::Buffer::new().format(offset_minute_print).bytes());
                    }
                }
                serializer.serialize_str(unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(dt.as_ptr(), dt.len()))
                })
            } else if unsafe { obj_ptr == DATE_PTR } {
                let mut dt: SmallVec<[u8; 10]> = SmallVec::with_capacity(10);
                {
                    let year = unsafe { pyo3::ffi::PyDateTime_GET_YEAR(self.ptr) as i32 };
                    dt.extend(itoa::Buffer::new().format(year).bytes());
                }
                dt.push(HYPHEN);

                {
                    let month = unsafe { pyo3::ffi::PyDateTime_GET_MONTH(self.ptr) as u32 };
                    if month < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(month).bytes());
                }
                dt.push(HYPHEN);
                {
                    let day = unsafe { pyo3::ffi::PyDateTime_GET_DAY(self.ptr) as u32 };
                    if day < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(day).bytes());
                }
                serializer.serialize_str(unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(dt.as_ptr(), dt.len()))
                })
            } else if unsafe { obj_ptr == TIME_PTR } {
                if unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_Time)).hastzinfo == 1 } {
                    return Err(ser::Error::custom("datetime.time must not have tzinfo set"));
                }
                let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
                {
                    let hour = unsafe { pyo3::ffi::PyDateTime_TIME_GET_HOUR(self.ptr) as u8 };
                    if hour < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(hour).bytes());

                    dt.push(COLON);
                }
                {
                    let minute = unsafe { pyo3::ffi::PyDateTime_TIME_GET_MINUTE(self.ptr) as u8 };
                    if minute < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(minute).bytes());

                    dt.push(COLON);
                }
                {
                    let second = unsafe { pyo3::ffi::PyDateTime_TIME_GET_SECOND(self.ptr) as u8 };
                    if second < 10 {
                        dt.push(ZERO);
                    }
                    dt.extend(itoa::Buffer::new().format(second).bytes());
                }
                {
                    let microsecond =
                        unsafe { pyo3::ffi::PyDateTime_TIME_GET_MICROSECOND(self.ptr) as u32 };
                    if microsecond != 0 {
                        dt.push(PERIOD);
                        dt.extend(itoa::Buffer::new().format(microsecond).bytes());
                    }
                }

                serializer.serialize_str(unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(dt.as_ptr(), dt.len()))
                })
            } else if self.default.is_some() {
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
                    } else {
                        Err(ser::Error::custom(format_args!(
                            "Type is not JSON serializable: {}",
                            unsafe { CStr::from_ptr((*obj_ptr).tp_name).to_string_lossy() }
                        )))
                    }
                }
            } else {
                Err(ser::Error::custom(format_args!(
                    "Type is not JSON serializable: {}",
                    unsafe { CStr::from_ptr((*obj_ptr).tp_name).to_string_lossy() }
                )))
            }
        }
    }
}
