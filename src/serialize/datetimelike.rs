use crate::opt::*;

#[derive(Debug)]
pub enum DateTimeError {
    LibraryUnsupported,
}

#[repr(transparent)]
pub struct DateTimeBuffer {
    buf: arrayvec::ArrayVec<u8, 32>,
}

impl DateTimeBuffer {
    pub fn new() -> DateTimeBuffer {
        DateTimeBuffer {
            buf: arrayvec::ArrayVec::<u8, 32>::new(),
        }
    }
    pub fn push(&mut self, value: u8) {
        self.buf.push(value);
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.buf.try_extend_from_slice(slice).unwrap();
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

macro_rules! write_double_digit {
    ($buf:ident, $value:expr) => {
        if $value < 10 {
            $buf.push(b'0');
        }
        $buf.extend_from_slice(itoa::Buffer::new().format($value).as_bytes());
    };
}

macro_rules! write_triple_digit {
    ($buf:ident, $value:expr) => {
        if $value < 100 {
            $buf.push(b'0');
        }
        if $value < 10 {
            $buf.push(b'0');
        }
        $buf.extend_from_slice(itoa::Buffer::new().format($value).as_bytes());
    };
}

#[derive(Default)]
pub struct Offset {
    pub day: i32,
    pub second: i32,
}

/// Trait providing a method to write a datetime-like object to a buffer in an RFC3339-compatible format.
///
/// The provided `write_buf` method does not allocate, and is faster
/// than writing to a heap-allocated string.
pub trait DateTimeLike {
    /// Returns the year component of the datetime.
    fn year(&self) -> i32;
    /// Returns the month component of the datetime.
    fn month(&self) -> u8;
    /// Returns the day component of the datetime.
    fn day(&self) -> u8;
    /// Returns the hour component of the datetime.
    fn hour(&self) -> u8;
    /// Returns the minute component of the datetime.
    fn minute(&self) -> u8;
    /// Returns the second component of the datetime.
    fn second(&self) -> u8;
    /// Returns the number of milliseconds since the whole non-leap second.
    fn millisecond(&self) -> u32;
    /// Returns the number of microseconds since the whole non-leap second.
    fn microsecond(&self) -> u32;
    /// Returns the number of nanoseconds since the whole non-leap second.
    fn nanosecond(&self) -> u32;

    /// Is the object time-zone aware?
    fn has_tz(&self) -> bool;

    //// python3.8 or below implementation of offset()
    fn slow_offset(&self) -> Result<Offset, DateTimeError>;

    /// The offset of the timezone.
    fn offset(&self) -> Result<Offset, DateTimeError>;

    /// Write `self` to a buffer in RFC3339 format, using `opts` to
    /// customise if desired.
    fn write_buf(&self, buf: &mut DateTimeBuffer, opts: Opt) -> Result<(), DateTimeError> {
        {
            let year = self.year();
            let mut yearbuf = itoa::Buffer::new();
            let formatted = yearbuf.format(year);
            if unlikely!(year < 1000) {
                // date-fullyear   = 4DIGIT
                buf.extend_from_slice(&[b'0', b'0', b'0', b'0'][..(4 - formatted.len())]);
            }
            buf.extend_from_slice(formatted.as_bytes());
        }
        buf.push(b'-');
        write_double_digit!(buf, self.month());
        buf.push(b'-');
        write_double_digit!(buf, self.day());
        buf.push(b'T');
        write_double_digit!(buf, self.hour());
        buf.push(b':');
        write_double_digit!(buf, self.minute());
        buf.push(b':');
        write_double_digit!(buf, self.second());
        if opts & OMIT_MICROSECONDS == 0 {
            let microsecond = self.microsecond();
            if microsecond != 0 {
                buf.push(b'.');
                write_triple_digit!(buf, microsecond / 1_000);
                write_triple_digit!(buf, microsecond % 1_000);
                // Don't support writing nanoseconds for now.
                // If requested, something like the following should work,
                // and the `DateTimeBuffer` type alias should be changed to
                // have length 35.
                // let nanosecond = self.nanosecond();
                // if nanosecond % 1_000 != 0 {
                //     write_triple_digit!(buf, nanosecond % 1_000);
                // }
            }
        }
        if self.has_tz() || opts & NAIVE_UTC != 0 {
            let offset = self.offset()?;
            let mut offset_second = offset.second;
            if offset_second == 0 {
                if opts & UTC_Z != 0 {
                    buf.push(b'Z');
                } else {
                    buf.extend_from_slice(&[b'+', b'0', b'0', b':', b'0', b'0']);
                }
            } else {
                // This branch is only really hit by the Python datetime implementation,
                // since numpy datetimes are all converted to UTC.
                if offset.day == -1 {
                    // datetime.timedelta(days=-1, seconds=68400) -> -05:00
                    buf.push(b'-');
                    offset_second = 86400 - offset_second;
                } else {
                    // datetime.timedelta(seconds=37800) -> +10:30
                    buf.push(b'+');
                }
                let offset_minute = offset_second / 60;
                let offset_hour = offset_minute / 60;
                write_double_digit!(buf, offset_hour);
                buf.push(b':');
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
                write_double_digit!(buf, offset_minute_print);
            }
        }
        Ok(())
    }
}
