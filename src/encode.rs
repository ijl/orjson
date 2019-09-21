// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::typeref::*;
use pyo3::prelude::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};
use smallvec::SmallVec;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub const STRICT_INTEGER: u8 = 1;
pub const NAIVE_UTC: u8 = 1 << 1;

pub const MAX_OPT: i8 = STRICT_INTEGER as i8 | NAIVE_UTC as i8;

const HYPHEN: u8 = 45; // "-"
const PLUS: u8 = 43; // "+"
const ZERO: u8 = 48; // "0"
const T: u8 = 84; // "T"
const COLON: u8 = 58; // ":"
const PERIOD: u8 = 46; // ":"

macro_rules! write_double_digit {
    ($dt:ident, $value:ident) => {
        if $value < 10 {
            $dt.push(ZERO);
        }
        $dt.extend(itoa::Buffer::new().format($value).bytes());
    };
}

macro_rules! write_microsecond {
    ($dt:ident, $microsecond:ident) => {
        if $microsecond != 0 {
            $dt.push(PERIOD);
            let mut buf = itoa::Buffer::new();
            let formatted = buf.format($microsecond);
            let mut to_pad = 6 - formatted.len();
            while to_pad != 0 {
                $dt.push(ZERO);
                to_pad -= 1;
            }
            $dt.extend(formatted.bytes());
        }
    };
}

macro_rules! obj_name {
    ($obj:ident) => {
        unsafe { CStr::from_ptr((*$obj).tp_name).to_string_lossy() }
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
                return Err(ser::Error::custom(INVALID_STR));
            }
            serializer.serialize_str(str_from_slice!(uni, str_size))
        } else if is_type!(obj_ptr, FLOAT_PTR) {
            serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr)))
        } else if is_type!(obj_ptr, INT_PTR) {
            let val = ffi!(PyLong_AsLongLong(self.ptr));
            if unlikely!(val == -1 && !pyo3::ffi::PyErr_Occurred().is_null()) {
                return Err(ser::Error::custom("Integer exceeds 64-bit range"));
            } else if self.opts & STRICT_INTEGER == STRICT_INTEGER
                && (val > STRICT_INT_MAX || val < STRICT_INT_MIN)
            {
                return Err(ser::Error::custom("Integer exceeds 53-bit range"));
            }
            serializer.serialize_i64(val)
        } else if is_type!(obj_ptr, BOOL_PTR) {
            serializer.serialize_bool(unsafe { self.ptr == TRUE })
        } else if is_type!(obj_ptr, NONE_PTR) {
            serializer.serialize_unit()
        } else if is_type!(obj_ptr, DICT_PTR) {
            let mut map = serializer.serialize_map(None).unwrap();
            let mut pos = 0isize;
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            while unsafe { pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0 } {
                if unlikely!(self.recursion == 255) {
                    return Err(ser::Error::custom("Recursion limit reached"));
                }
                if unlikely!((*key).ob_type != STR_PTR) {
                    return Err(ser::Error::custom("Dict key must be str"));
                }
                let data = ffi!(PyUnicode_AsUTF8AndSize(key, &mut str_size)) as *const u8;
                if unlikely!(data.is_null()) {
                    return Err(ser::Error::custom(INVALID_STR));
                }
                map.serialize_entry(
                    str_from_slice!(data, str_size),
                    &SerializePyObject {
                        ptr: value,
                        default: self.default,
                        opts: self.opts,
                        default_calls: self.default_calls,
                        recursion: self.recursion + 1,
                    },
                )?;
            }
            map.end()
        } else if is_type!(obj_ptr, LIST_PTR) {
            let len = ffi!(PyList_GET_SIZE(self.ptr)) as usize;
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                let mut i = 0;
                while i < len {
                    if unlikely!(self.recursion == 255) {
                        return Err(ser::Error::custom("Recursion limit reached"));
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
        } else {
            if is_type!(obj_ptr, TUPLE_PTR) {
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
                let has_tz =
                    unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 1 };
                let offset_day: i32;
                let mut offset_second: i32;
                if !has_tz {
                    offset_second = 0;
                    offset_day = 0;
                } else {
                    let tzinfo = ffi!(PyDateTime_DATE_GET_TZINFO(self.ptr));
                    if unsafe {
                        (*(self.ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 1
                    } {
                        if ffi!(PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR)) == 1 {
                            // pendulum
                            let offset = unsafe {
                                pyo3::ffi::PyObject_CallMethodObjArgs(
                                    self.ptr,
                                    UTCOFFSET_METHOD_STR,
                                    std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                                )
                            };
                            // test_datetime_partial_second_pendulum_not_supported
                            if offset.is_null() {
                                return Err(ser::Error::custom(
                                        "datetime does not support timezones with offsets that are not even minutes",
                                    ));
                            }
                            offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                            offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                        } else if unsafe {
                            pyo3::ffi::PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR) == 1
                        } {
                            // pytz
                            let offset = unsafe {
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
                            offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                            offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                        } else if ffi!(PyObject_HasAttr(tzinfo, DST_STR)) == 1 {
                            // dateutil/arrow, datetime.timezone.utc
                            let offset = unsafe {
                                pyo3::ffi::PyObject_CallMethodObjArgs(
                                    tzinfo,
                                    UTCOFFSET_METHOD_STR,
                                    self.ptr,
                                    std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                                )
                            };
                            offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                            offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                        } else {
                            return Err(ser::Error::custom(
                        "datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil",
                    ));
                        }
                    } else {
                        offset_second = 0;
                        offset_day = 0;
                    }
                };

                let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
                dt.extend(
                    itoa::Buffer::new()
                        .format(ffi!(PyDateTime_GET_YEAR(self.ptr)) as i32)
                        .bytes(),
                );
                dt.push(HYPHEN);
                {
                    let month = ffi!(PyDateTime_GET_MONTH(self.ptr)) as u8;
                    write_double_digit!(dt, month);
                }
                dt.push(HYPHEN);
                {
                    let day = ffi!(PyDateTime_GET_DAY(self.ptr)) as u8;
                    write_double_digit!(dt, day);
                }
                dt.push(T);
                {
                    let hour = ffi!(PyDateTime_DATE_GET_HOUR(self.ptr)) as u8;
                    write_double_digit!(dt, hour);
                }
                dt.push(COLON);
                {
                    let minute = ffi!(PyDateTime_DATE_GET_MINUTE(self.ptr)) as u8;
                    write_double_digit!(dt, minute);
                }
                dt.push(COLON);
                {
                    let second = ffi!(PyDateTime_DATE_GET_SECOND(self.ptr)) as u8;
                    write_double_digit!(dt, second);
                }
                {
                    let microsecond = ffi!(PyDateTime_DATE_GET_MICROSECOND(self.ptr)) as u32;
                    write_microsecond!(dt, microsecond);
                }
                if has_tz || self.opts & NAIVE_UTC == NAIVE_UTC {
                    if offset_second == 0 {
                        dt.extend([PLUS, ZERO, ZERO, COLON, ZERO, ZERO].iter().cloned());
                    } else {
                        if offset_day == -1 {
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
                }
                serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len()))
            } else if is_type!(obj_ptr, DATE_PTR) {
                let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
                {
                    let year = ffi!(PyDateTime_GET_YEAR(self.ptr)) as i32;
                    dt.extend(itoa::Buffer::new().format(year).bytes());
                }
                dt.push(HYPHEN);
                {
                    let month = ffi!(PyDateTime_GET_MONTH(self.ptr)) as u32;
                    write_double_digit!(dt, month);
                }
                dt.push(HYPHEN);
                {
                    let day = ffi!(PyDateTime_GET_DAY(self.ptr)) as u32;
                    write_double_digit!(dt, day);
                }
                serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len()))
            } else if is_type!(obj_ptr, TIME_PTR) {
                if unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_Time)).hastzinfo == 1 } {
                    return Err(ser::Error::custom("datetime.time must not have tzinfo set"));
                }
                let mut dt: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
                {
                    let hour = ffi!(PyDateTime_TIME_GET_HOUR(self.ptr)) as u8;
                    write_double_digit!(dt, hour);
                }
                dt.push(COLON);
                {
                    let minute = ffi!(PyDateTime_TIME_GET_MINUTE(self.ptr)) as u8;
                    write_double_digit!(dt, minute);
                }
                dt.push(COLON);
                {
                    let second = ffi!(PyDateTime_TIME_GET_SECOND(self.ptr)) as u8;
                    write_double_digit!(dt, second);
                }
                {
                    let microsecond = ffi!(PyDateTime_TIME_GET_MICROSECOND(self.ptr)) as u32;
                    write_microsecond!(dt, microsecond);
                }
                serializer.serialize_str(str_from_slice!(dt.as_ptr(), dt.len()))
            } else if self.default.is_some() {
                if self.default_calls > 5 {
                    Err(ser::Error::custom(
                        "default serializer exceeds recursion limit",
                    ))
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
                            opts: self.opts,
                            default_calls: self.default_calls + 1,
                            recursion: self.recursion,
                        }
                        .serialize(serializer);
                        ffi!(Py_DECREF(default_obj));
                        res
                    } else if !ffi!(PyErr_Occurred()).is_null() {
                        Err(ser::Error::custom(format_args!(
                            "Type raised exception in default function: {}",
                            obj_name!(obj_ptr)
                        )))
                    } else {
                        Err(ser::Error::custom(format_args!(
                            "Type is not JSON serializable: {}",
                            obj_name!(obj_ptr)
                        )))
                    }
                }
            } else {
                Err(ser::Error::custom(format_args!(
                    "Type is not JSON serializable: {}",
                    obj_name!(obj_ptr)
                )))
            }
        }
    }
}
