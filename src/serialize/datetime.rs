// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::datetimelike::{DateTimeBuffer, DateTimeError, DateTimeLike, Offset};
use crate::serialize::error::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};

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
    ptr: *mut pyo3_ffi::PyObject,
}

impl Date {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        Date { ptr: ptr }
    }
    pub fn write_buf(&self, buf: &mut DateTimeBuffer) {
        {
            let year = ffi!(PyDateTime_GET_YEAR(self.ptr));
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
impl Serialize for Date {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = DateTimeBuffer::new();
        self.write_buf(&mut buf);
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub enum TimeError {
    HasTimezone,
}

pub struct Time {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
}

impl Time {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, opts: Opt) -> Self {
        Time {
            ptr: ptr,
            opts: opts,
        }
    }
    pub fn write_buf(&self, buf: &mut DateTimeBuffer) -> Result<(), TimeError> {
        if unsafe { (*(self.ptr as *mut pyo3_ffi::PyDateTime_Time)).hastzinfo == 1 } {
            return Err(TimeError::HasTimezone);
        }
        let hour = ffi!(PyDateTime_TIME_GET_HOUR(self.ptr)) as u8;
        write_double_digit!(buf, hour);
        buf.push(b':');
        let minute = ffi!(PyDateTime_TIME_GET_MINUTE(self.ptr)) as u8;
        write_double_digit!(buf, minute);
        buf.push(b':');
        let second = ffi!(PyDateTime_TIME_GET_SECOND(self.ptr)) as u8;
        write_double_digit!(buf, second);
        if self.opts & OMIT_MICROSECONDS == 0 {
            let microsecond = ffi!(PyDateTime_TIME_GET_MICROSECOND(self.ptr)) as u32;
            write_microsecond!(buf, microsecond);
        }
        Ok(())
    }
}

impl Serialize for Time {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = DateTimeBuffer::new();
        if self.write_buf(&mut buf).is_err() {
            err!(SerializeError::DatetimeLibraryUnsupported)
        };
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub struct DateTime {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
}

impl DateTime {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, opts: Opt) -> Self {
        DateTime {
            ptr: ptr,
            opts: opts,
        }
    }
}

macro_rules! pydatetime_get {
    ($fn: ident, $pyfn: ident, $ty: ident) => {
        fn $fn(&self) -> $ty {
            ffi!($pyfn(self.ptr)) as $ty
        }
    };
}

impl DateTimeLike for DateTime {
    pydatetime_get!(year, PyDateTime_GET_YEAR, i32);
    pydatetime_get!(month, PyDateTime_GET_MONTH, u8);
    pydatetime_get!(day, PyDateTime_GET_DAY, u8);
    pydatetime_get!(hour, PyDateTime_DATE_GET_HOUR, u8);
    pydatetime_get!(minute, PyDateTime_DATE_GET_MINUTE, u8);
    pydatetime_get!(second, PyDateTime_DATE_GET_SECOND, u8);
    pydatetime_get!(microsecond, PyDateTime_DATE_GET_MICROSECOND, u32);

    fn millisecond(&self) -> u32 {
        self.microsecond() / 1_000
    }

    fn nanosecond(&self) -> u32 {
        self.microsecond() * 1_000
    }

    fn has_tz(&self) -> bool {
        unsafe { (*(self.ptr as *mut pyo3_ffi::PyDateTime_DateTime)).hastzinfo == 1 }
    }

    fn slow_offset(&self) -> Result<Offset, DateTimeError> {
        let tzinfo = ffi!(PyDateTime_DATE_GET_TZINFO(self.ptr));
        if ffi!(PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR)) == 1 {
            // pendulum
            let py_offset = call_method!(self.ptr, UTCOFFSET_METHOD_STR);
            let offset = Offset {
                second: ffi!(PyDateTime_DELTA_GET_SECONDS(py_offset)),
                day: ffi!(PyDateTime_DELTA_GET_DAYS(py_offset)),
            };
            ffi!(Py_DECREF(py_offset));
            Ok(offset)
        } else if ffi!(PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR)) == 1 {
            // pytz
            let method_ptr = call_method!(tzinfo, NORMALIZE_METHOD_STR, self.ptr);
            let py_offset = call_method!(method_ptr, UTCOFFSET_METHOD_STR);
            ffi!(Py_DECREF(method_ptr));
            let offset = Offset {
                second: ffi!(PyDateTime_DELTA_GET_SECONDS(py_offset)),
                day: ffi!(PyDateTime_DELTA_GET_DAYS(py_offset)),
            };
            ffi!(Py_DECREF(py_offset));
            Ok(offset)
        } else if ffi!(PyObject_HasAttr(tzinfo, DST_STR)) == 1 {
            // dateutil/arrow, datetime.timezone.utc
            let py_offset = call_method!(tzinfo, UTCOFFSET_METHOD_STR, self.ptr);
            let offset = Offset {
                second: ffi!(PyDateTime_DELTA_GET_SECONDS(py_offset)),
                day: ffi!(PyDateTime_DELTA_GET_DAYS(py_offset)),
            };
            ffi!(Py_DECREF(py_offset));
            Ok(offset)
        } else {
            Err(DateTimeError::LibraryUnsupported)
        }
    }

    #[cfg(Py_3_9)]
    fn offset(&self) -> Result<Offset, DateTimeError> {
        if !self.has_tz() {
            Ok(Offset::default())
        } else {
            let tzinfo = ffi!(PyDateTime_DATE_GET_TZINFO(self.ptr));
            if unsafe { ob_type!(tzinfo) == ZONEINFO_TYPE } {
                // zoneinfo
                let py_offset = call_method!(tzinfo, UTCOFFSET_METHOD_STR, self.ptr);
                let offset = Offset {
                    second: ffi!(PyDateTime_DELTA_GET_SECONDS(py_offset)),
                    day: ffi!(PyDateTime_DELTA_GET_DAYS(py_offset)),
                };
                ffi!(Py_DECREF(py_offset));
                Ok(offset)
            } else {
                self.slow_offset()
            }
        }
    }

    #[cfg(not(Py_3_9))]
    fn offset(&self) -> Result<Offset, DateTimeError> {
        if !self.has_tz() {
            Ok(Offset::default())
        } else {
            self.slow_offset()
        }
    }
}

impl Serialize for DateTime {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = DateTimeBuffer::new();
        if self.write_buf(&mut buf, self.opts).is_err() {
            err!(SerializeError::DatetimeLibraryUnsupported)
        }
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
