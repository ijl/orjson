// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::opt::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};

pub type DateTimeBuffer = smallvec::SmallVec<[u8; 32]>;

macro_rules! write_double_digit {
    ($buf:ident, $value:ident) => {
        if $value < 10 {
            $buf.push(b'0');
        }
        $buf.extend_from_slice(itoa::Buffer::new().format($value).as_bytes());
    };
}

macro_rules! write_microsecond {
    ($buf:ident, $microsecond:ident) => {
        if $microsecond != 0 {
            let mut buf = itoa::Buffer::new();
            let formatted = buf.format($microsecond);
            $buf.extend_from_slice(
                &[b'.', b'0', b'0', b'0', b'0', b'0', b'0'][..(7 - formatted.len())],
            );
            $buf.extend_from_slice(formatted.as_bytes());
        }
    };
}

#[repr(transparent)]
pub struct Date {
    ptr: *mut pyo3::ffi::PyObject,
}

impl Date {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        Date { ptr: ptr }
    }
    pub fn write_buf(&self, buf: &mut DateTimeBuffer) {
        {
            let year = ffi!(PyDateTime_GET_YEAR(self.ptr)) as i32;
            let mut yearbuf = itoa::Buffer::new();
            let formatted = yearbuf.format(year);
            if unlikely!(year < 1000) {
                // date-fullyear   = 4DIGIT
                buf.extend_from_slice(&[b'0', b'0', b'0', b'0'][..(4 - formatted.len())]);
            }
            buf.extend_from_slice(formatted.as_bytes());
        }
        buf.push(b'-');
        {
            let month = ffi!(PyDateTime_GET_MONTH(self.ptr)) as u32;
            write_double_digit!(buf, month);
        }
        buf.push(b'-');
        {
            let day = ffi!(PyDateTime_GET_DAY(self.ptr)) as u32;
            write_double_digit!(buf, day);
        }
    }
}
impl<'p> Serialize for Date {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
        self.write_buf(&mut buf);
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub enum TimeError {
    HasTimezone,
}

pub struct Time {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl Time {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Result<Self, TimeError> {
        if unsafe { (*(ptr as *mut pyo3::ffi::PyDateTime_Time)).hastzinfo == 1 } {
            return Err(TimeError::HasTimezone);
        }
        Ok(Time {
            ptr: ptr,
            opts: opts,
        })
    }
    pub fn write_buf(&self, buf: &mut DateTimeBuffer) {
        {
            let hour = ffi!(PyDateTime_TIME_GET_HOUR(self.ptr)) as u8;
            write_double_digit!(buf, hour);
        }
        buf.push(b':');
        {
            let minute = ffi!(PyDateTime_TIME_GET_MINUTE(self.ptr)) as u8;
            write_double_digit!(buf, minute);
        }
        buf.push(b':');
        {
            let second = ffi!(PyDateTime_TIME_GET_SECOND(self.ptr)) as u8;
            write_double_digit!(buf, second);
        }
        if self.opts & OMIT_MICROSECONDS == 0 {
            let microsecond = ffi!(PyDateTime_TIME_GET_MICROSECOND(self.ptr)) as u32;
            write_microsecond!(buf, microsecond);
        }
    }
}

impl<'p> Serialize for Time {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
        self.write_buf(&mut buf);
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub enum DateTimeError {
    LibraryUnsupported,
}

pub struct DateTime {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl DateTime {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Self {
        DateTime {
            ptr: ptr,
            opts: opts,
        }
    }
    pub fn write_buf(&self, buf: &mut DateTimeBuffer) -> Result<(), DateTimeError> {
        let has_tz = unsafe { (*(self.ptr as *mut pyo3::ffi::PyDateTime_DateTime)).hastzinfo == 1 };
        let offset_day: i32;
        let mut offset_second: i32;
        if !has_tz {
            offset_second = 0;
            offset_day = 0;
        } else {
            let tzinfo = ffi!(PyDateTime_DATE_GET_TZINFO(self.ptr));
            if ffi!(PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR)) == 1 {
                // pendulum
                let offset = call_method!(self.ptr, UTCOFFSET_METHOD_STR);
                offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                ffi!(Py_DECREF(offset));
            } else if ffi!(PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR)) == 1 {
                // pytz
                let method_ptr = call_method!(tzinfo, NORMALIZE_METHOD_STR, self.ptr);
                let offset = call_method!(method_ptr, UTCOFFSET_METHOD_STR);
                ffi!(Py_DECREF(method_ptr));
                offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                ffi!(Py_DECREF(offset));
            } else if ffi!(PyObject_HasAttr(tzinfo, DST_STR)) == 1 {
                // dateutil/arrow, datetime.timezone.utc
                let offset = call_method!(tzinfo, UTCOFFSET_METHOD_STR, self.ptr);
                offset_second = ffi!(PyDateTime_DELTA_GET_SECONDS(offset)) as i32;
                offset_day = ffi!(PyDateTime_DELTA_GET_DAYS(offset));
                ffi!(Py_DECREF(offset));
            } else {
                return Err(DateTimeError::LibraryUnsupported);
            }
        };
        {
            let year = ffi!(PyDateTime_GET_YEAR(self.ptr)) as i32;
            let mut yearbuf = itoa::Buffer::new();
            let formatted = yearbuf.format(year);
            if unlikely!(year < 1000) {
                // date-fullyear   = 4DIGIT
                buf.extend_from_slice(&[b'0', b'0', b'0', b'0'][..(4 - formatted.len())]);
            }
            buf.extend_from_slice(formatted.as_bytes());
        }
        buf.push(b'-');
        {
            let month = ffi!(PyDateTime_GET_MONTH(self.ptr)) as u8;
            write_double_digit!(buf, month);
        }
        buf.push(b'-');
        {
            let day = ffi!(PyDateTime_GET_DAY(self.ptr)) as u8;
            write_double_digit!(buf, day);
        }
        buf.push(b'T');
        {
            let hour = ffi!(PyDateTime_DATE_GET_HOUR(self.ptr)) as u8;
            write_double_digit!(buf, hour);
        }
        buf.push(b':');
        {
            let minute = ffi!(PyDateTime_DATE_GET_MINUTE(self.ptr)) as u8;
            write_double_digit!(buf, minute);
        }
        buf.push(b':');
        {
            let second = ffi!(PyDateTime_DATE_GET_SECOND(self.ptr)) as u8;
            write_double_digit!(buf, second);
        }
        if self.opts & OMIT_MICROSECONDS == 0 {
            let microsecond = ffi!(PyDateTime_DATE_GET_MICROSECOND(self.ptr)) as u32;
            write_microsecond!(buf, microsecond);
        }
        if has_tz || self.opts & NAIVE_UTC != 0 {
            if offset_second == 0 {
                if self.opts & UTC_Z != 0 {
                    buf.push(b'Z');
                } else {
                    buf.extend_from_slice(&[b'+', b'0', b'0', b':', b'0', b'0']);
                }
            } else {
                if offset_day == -1 {
                    // datetime.timedelta(days=-1, seconds=68400) -> -05:00
                    buf.push(b'-');
                    offset_second = 86400 - offset_second;
                } else {
                    // datetime.timedelta(seconds=37800) -> +10:30
                    buf.push(b'+');
                }
                {
                    let offset_minute = offset_second / 60;
                    let offset_hour = offset_minute / 60;
                    write_double_digit!(buf, offset_hour);
                    buf.push(b':');

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
                        buf.push(b'0');
                    }
                    buf.extend_from_slice(
                        itoa::Buffer::new().format(offset_minute_print).as_bytes(),
                    );
                }
            }
        }
        Ok(())
    }
}

impl<'p> Serialize for DateTime {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf: DateTimeBuffer = smallvec::SmallVec::with_capacity(32);
        if self.write_buf(&mut buf).is_err() {
            err!(DATETIME_LIBRARY_UNSUPPORTED)
        }
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
