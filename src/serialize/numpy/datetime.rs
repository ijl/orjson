// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::{NumpyDateTimeError, NumpyDatetime64Repr, PyStrRef};
use crate::opt::{NAIVE_UTC, OMIT_MICROSECONDS, UTC_Z};
use crate::serialize::{
    error::SerializeError,
    writer::{SmallFixedBuffer, WriteExt, write_integer_i64, write_integer_u32},
};

pub(crate) fn datetime_into_error(val: NumpyDateTimeError) -> SerializeError {
    let err = match val {
        NumpyDateTimeError::UnsupportedUnit(unit) => {
            let mut msg = String::from("unsupported numpy.datetime64 unit: ");
            msg.push_str(unit.as_str());
            msg
        }
        NumpyDateTimeError::Unrepresentable { unit, val } => {
            let mut buf = SmallFixedBuffer::new();
            write_integer_i64(&mut buf, val);
            let val_str = str_from_slice!(buf.as_ptr(), buf.len());

            let mut msg = String::from("unrepresentable numpy.datetime64: ");
            msg.push_str(val_str);
            msg.push(' ');
            msg.push_str(unit.as_str());
            msg
        }
    };
    SerializeError::NumpyUnsupportedDatetimeUnit(PyStrRef::from_str(&err))
}

pub(crate) fn write_numpy_datetime<B>(ob: &NumpyDatetime64Repr, buf: &mut B)
where
    B: ?Sized + WriteExt + bytes::BufMut,
{
    {
        // buf.put_u8(b'"');
        let year = ob.year();
        if year < 1000 {
            cold_path!();
            // date-fullyear   = 4DIGIT
            buf.put_u8(b'0');
            if year < 100 {
                buf.put_u8(b'0');
            }
            if year < 10 {
                buf.put_u8(b'0');
            }
        }
        write_integer_u32(buf, year.cast_unsigned());
    }
    buf.put_u8(b'-');
    write_double_digit!(buf, ob.month() as u32);
    buf.put_u8(b'-');
    write_double_digit!(buf, ob.day() as u32);
    buf.put_u8(b'T');
    write_double_digit!(buf, ob.hour() as u32);
    buf.put_u8(b':');
    write_double_digit!(buf, ob.minute() as u32);
    buf.put_u8(b':');
    write_double_digit!(buf, ob.second() as u32);
    if opt_disabled!(ob.opts, OMIT_MICROSECONDS) {
        let microsecond = ob.microsecond();
        if microsecond != 0 {
            buf.put_u8(b'.');
            write_triple_digit!(buf, microsecond / 1_000);
            write_triple_digit!(buf, microsecond % 1_000);
        }
    }
    if opt_enabled!(ob.opts, NAIVE_UTC) {
        if opt_enabled!(ob.opts, UTC_Z) {
            buf.put_u8(b'Z');
        } else {
            buf.put_slice(b"+00:00");
        }
    }
    // buf.put_u8(b'"');
}
