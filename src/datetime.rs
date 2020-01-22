// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use smallvec::SmallVec;

pub const NAIVE_UTC: u8 = 1 << 1;
pub const OMIT_MICROSECONDS: u8 = 1 << 2;
pub const UTC_Z: u8 = 1 << 3;

pub const HYPHEN: u8 = 45; // "-"
const PLUS: u8 = 43; // "+"
const ZERO: u8 = 48; // "0"
const T: u8 = 84; // "T"
const COLON: u8 = 58; // ":"
const PERIOD: u8 = 46; // ":"
const Z: u8 = 90; // "Z"

macro_rules! write_double_digit {
    ($dt:ident, $value:ident) => {
        if $value < 10 {
            $dt.push(ZERO);
        }
        $dt.extend_from_slice(itoa::Buffer::new().format($value).as_bytes());
    };
}

macro_rules! write_microsecond {
    ($dt:ident, $microsecond:ident) => {
        if $microsecond != 0 {
            $dt.push(PERIOD);
            let mut buf = itoa::Buffer::new();
            let formatted = buf.format($microsecond);
            $dt.extend_from_slice(&[ZERO; 6][..(6 - formatted.len())]);
            $dt.extend_from_slice(formatted.as_bytes());
        }
    };
}

pub enum DatetimeError {
    Library,
}

#[inline(never)]
pub fn write_datetime(
    ptr: *mut pyo3::ffi::PyObject,
    opts: u8,
    dt: &mut SmallVec<[u8; 32]>,
) -> Result<(), DatetimeError> {
    let has_tz = unsafe { (*(ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 1 };
    let offset_day: i32;
    let mut offset_second: i32;
    if !has_tz {
        offset_second = 0;
        offset_day = 0;
    } else {
        let tzinfo = ffi!(PyDateTime_DATE_GET_TZINFO(ptr));
        if unsafe { (*(ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 1 } {
            if ffi!(PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR)) == 1 {
                // pendulum
                let offset = unsafe {
                    pyo3::ffi::PyObject_CallMethodObjArgs(
                        ptr,
                        UTCOFFSET_METHOD_STR,
                        std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                    )
                };
                offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
            } else if ffi!(PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR)) == 1 {
                // pytz
                let offset = unsafe {
                    pyo3::ffi::PyObject_CallMethodObjArgs(
                        pyo3::ffi::PyObject_CallMethodObjArgs(
                            tzinfo,
                            NORMALIZE_METHOD_STR,
                            ptr,
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
                        ptr,
                        std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                    )
                };
                offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
            } else {
                return Err(DatetimeError::Library);
            }
        } else {
            offset_second = 0;
            offset_day = 0;
        }
    };

    dt.extend_from_slice(
        itoa::Buffer::new()
            .format(ffi!(PyDateTime_GET_YEAR(ptr)) as i32)
            .as_bytes(),
    );
    dt.push(HYPHEN);
    {
        let month = ffi!(PyDateTime_GET_MONTH(ptr)) as u8;
        write_double_digit!(dt, month);
    }
    dt.push(HYPHEN);
    {
        let day = ffi!(PyDateTime_GET_DAY(ptr)) as u8;
        write_double_digit!(dt, day);
    }
    dt.push(T);
    {
        let hour = ffi!(PyDateTime_DATE_GET_HOUR(ptr)) as u8;
        write_double_digit!(dt, hour);
    }
    dt.push(COLON);
    {
        let minute = ffi!(PyDateTime_DATE_GET_MINUTE(ptr)) as u8;
        write_double_digit!(dt, minute);
    }
    dt.push(COLON);
    {
        let second = ffi!(PyDateTime_DATE_GET_SECOND(ptr)) as u8;
        write_double_digit!(dt, second);
    }
    if opts & OMIT_MICROSECONDS != OMIT_MICROSECONDS {
        let microsecond = ffi!(PyDateTime_DATE_GET_MICROSECOND(ptr)) as u32;
        write_microsecond!(dt, microsecond);
    }
    if has_tz || opts & NAIVE_UTC == NAIVE_UTC {
        if offset_second == 0 {
            if opts & UTC_Z == UTC_Z {
                dt.push(Z);
            } else {
                dt.extend_from_slice(&[PLUS, ZERO, ZERO, COLON, ZERO, ZERO]);
            }
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
                dt.extend_from_slice(itoa::Buffer::new().format(offset_hour).as_bytes());
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
                dt.extend_from_slice(itoa::Buffer::new().format(offset_minute_print).as_bytes());
            }
        }
    }
    Ok(())
}

#[inline(never)]
pub fn write_date(ptr: *mut pyo3::ffi::PyObject, dt: &mut SmallVec<[u8; 32]>) {
    {
        let year = ffi!(PyDateTime_GET_YEAR(ptr)) as i32;
        dt.extend_from_slice(itoa::Buffer::new().format(year).as_bytes());
    }
    dt.push(HYPHEN);
    {
        let month = ffi!(PyDateTime_GET_MONTH(ptr)) as u32;
        write_double_digit!(dt, month);
    }
    dt.push(HYPHEN);
    {
        let day = ffi!(PyDateTime_GET_DAY(ptr)) as u32;
        write_double_digit!(dt, day);
    }
}

#[inline(never)]
pub fn write_time(ptr: *mut pyo3::ffi::PyObject, opts: u8, dt: &mut SmallVec<[u8; 32]>) {
    {
        let hour = ffi!(PyDateTime_TIME_GET_HOUR(ptr)) as u8;
        write_double_digit!(dt, hour);
    }
    dt.push(COLON);
    {
        let minute = ffi!(PyDateTime_TIME_GET_MINUTE(ptr)) as u8;
        write_double_digit!(dt, minute);
    }
    dt.push(COLON);
    {
        let second = ffi!(PyDateTime_TIME_GET_SECOND(ptr)) as u8;
        write_double_digit!(dt, second);
    }
    if opts & OMIT_MICROSECONDS != OMIT_MICROSECONDS {
        let microsecond = ffi!(PyDateTime_TIME_GET_MICROSECOND(ptr)) as u32;
        write_microsecond!(dt, microsecond);
    }
}
