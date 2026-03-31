// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2025-2026)

use crate::ffi::{PyDateRef, PyDateTimeRef, PyTimeRef};
use crate::opt::{NAIVE_UTC, OMIT_MICROSECONDS, Opt, UTC_Z};
use crate::serialize::{
    error::SerializeError,
    writer::{WriteExt, write_integer_u32},
};

#[cold]
#[inline(never)]
pub(crate) fn write_time<B>(ob: PyTimeRef, opts: Opt, buf: &mut B) -> Result<(), SerializeError>
where
    B: ?Sized + WriteExt + bytes::BufMut,
{
    if ob.has_tz() {
        return Err(SerializeError::TimeHasTzinfo);
    }
    write_double_digit!(buf, ob.hour() as u32);
    buf.put_u8(b':');
    write_double_digit!(buf, ob.minute() as u32);
    buf.put_u8(b':');
    write_double_digit!(buf, ob.second() as u32);
    if opt_disabled!(opts, OMIT_MICROSECONDS) {
        write_microsecond!(buf, ob.microsecond());
    }
    Ok(())
}

#[cold]
#[inline(never)]
pub(crate) fn write_date<B>(ob: PyDateRef, buf: &mut B)
where
    B: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
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
        write_integer_u32(buf, year);
        buf.put_u8(b'-');
        write_double_digit!(buf, ob.month());
        buf.put_u8(b'-');
        write_double_digit!(buf, ob.day());
    }
}

#[inline(never)]
pub(crate) fn write_datetime<B>(
    ob: PyDateTimeRef,
    opts: Opt,
    buf: &mut B,
) -> Result<(), SerializeError>
where
    B: ?Sized + WriteExt + bytes::BufMut,
{
    {
        let year = ob.year();
        debug_assert!(year >= 0);
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
    if opt_disabled!(opts, OMIT_MICROSECONDS) {
        let microsecond = ob.microsecond();
        if microsecond != 0 {
            buf.put_u8(b'.');
            write_triple_digit!(buf, microsecond / 1_000);
            write_triple_digit!(buf, microsecond % 1_000);
        }
    }
    if ob.has_tz() || opt_enabled!(opts, NAIVE_UTC) {
        match ob.offset() {
            Some(offset) => {
                let mut offset_second = offset.second;
                if offset_second == 0 {
                    if opt_enabled!(opts, UTC_Z) {
                        buf.put_u8(b'Z');
                    } else {
                        buf.put_slice(b"+00:00");
                    }
                } else {
                    if offset.day == -1 {
                        // datetime.timedelta(days=-1, seconds=68400) -> -05:00
                        buf.put_u8(b'-');
                        offset_second = 86400 - offset_second;
                    } else {
                        // datetime.timedelta(seconds=37800) -> +10:30
                        buf.put_u8(b'+');
                    }
                    let offset_minute = offset_second / 60;
                    let offset_hour = offset_minute / 60;
                    write_double_digit!(buf, offset_hour.cast_unsigned());
                    buf.put_u8(b':');
                    let mut offset_minute_print = offset_minute % 60;
                    // https://tools.ietf.org/html/rfc3339#section-5.8
                    // "exactly 19 minutes and 32.13 seconds ahead of UTC"
                    // "closest representable UTC offset"
                    //  "+20:00"
                    let offset_excess_second =
                        offset_second - (offset_minute_print * 60 + offset_hour * 3600);
                    if offset_excess_second >= 30 {
                        offset_minute_print += 1;
                    }
                    write_double_digit!(buf, offset_minute_print.cast_unsigned());
                }
            }
            None => {
                return Err(SerializeError::DatetimeLibraryUnsupported);
            }
        }
    }
    Ok(())
}
