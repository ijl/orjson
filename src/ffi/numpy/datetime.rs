// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2022-2026), Ben Sully (2021)

use crate::ffi::{Py_DECREF, PyListRef, PyObject, PyObject_GetAttr, PyStrRef, PyTupleRef};
use crate::opt::Opt;
use crate::typeref::{DESCR_STR, DTYPE_STR};
use jiff::Timestamp;
use jiff::civil::DateTime;

/// This mimicks the units supported by numpy's datetime64 type.
///
/// See
/// https://github.com/numpy/numpy/blob/fc8e3bbe419748ac5c6b7f3d0845e4bafa74644b/numpy/core/include/numpy/ndarraytypes.h#L268-L282.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NumpyDatetimeUnit {
    NaT,
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
    Femtoseconds,
    Attoseconds,
    Generic,
}

impl NumpyDatetimeUnit {
    #[cold]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NaT => "NaT",
            Self::Years => "years",
            Self::Months => "months",
            Self::Weeks => "weeks",
            Self::Days => "days",
            Self::Hours => "hours",
            Self::Minutes => "minutes",
            Self::Seconds => "seconds",
            Self::Milliseconds => "milliseconds",
            Self::Microseconds => "microseconds",
            Self::Nanoseconds => "nanoseconds",
            Self::Picoseconds => "picoseconds",
            Self::Femtoseconds => "femtoseconds",
            Self::Attoseconds => "attoseconds",
            Self::Generic => "generic",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum NumpyDateTimeError {
    UnsupportedUnit(NumpyDatetimeUnit),
    Unrepresentable { unit: NumpyDatetimeUnit, val: i64 },
}

macro_rules! to_jiff_datetime {
    ($timestamp:expr, $self:expr, $val:expr) => {
        Ok(
            ($timestamp.map_err(|_| NumpyDateTimeError::Unrepresentable {
                unit: $self,
                val: $val,
            })?)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .datetime(),
        )
    };
}

impl NumpyDatetimeUnit {
    /// Create a `NumpyDatetimeUnit` from a pointer to a Python object holding a
    /// numpy array.
    ///
    /// This function must only be called with pointers to numpy arrays.
    ///
    /// We need to look inside the `obj.dtype.descr` attribute of the Python
    /// object rather than using the `descr` field of the `__array_struct__`
    /// because that field isn't populated for datetime64 arrays; see
    /// https://github.com/numpy/numpy/issues/5350.
    #[cold]
    #[inline(never)]
    pub fn from_pyobject(ptr: *mut PyObject) -> Self {
        let dtype = unsafe { PyObject_GetAttr(ptr, DTYPE_STR) };
        let descr = unsafe { PyObject_GetAttr(dtype, DESCR_STR) };
        let el0 = unsafe { PyListRef::from_ptr_unchecked(descr).get(0) };
        let descr_str = unsafe { PyTupleRef::from_ptr_unchecked(el0).get(1) };
        match PyStrRef::from_ptr(descr_str) {
            Ok(uni) => {
                match uni.as_str() {
                    Some(as_str) => {
                        if as_str.len() < 5 {
                            return Self::NaT;
                        }
                        // unit descriptions are found at
                        // https://github.com/numpy/numpy/blob/b235f9e701e14ed6f6f6dcba885f7986a833743f/numpy/core/src/multiarray/datetime.c#L79-L96.
                        let ret = match &as_str[4..as_str.len() - 1] {
                            "Y" => Self::Years,
                            "M" => Self::Months,
                            "W" => Self::Weeks,
                            "D" => Self::Days,
                            "h" => Self::Hours,
                            "m" => Self::Minutes,
                            "s" => Self::Seconds,
                            "ms" => Self::Milliseconds,
                            "us" => Self::Microseconds,
                            "ns" => Self::Nanoseconds,
                            "ps" => Self::Picoseconds,
                            "fs" => Self::Femtoseconds,
                            "as" => Self::Attoseconds,
                            "generic" => Self::Generic,
                            _ => unreachable!(),
                        };
                        unsafe {
                            Py_DECREF(dtype);
                            Py_DECREF(descr);
                        };
                        ret
                    }
                    None => Self::NaT,
                }
            }
            Err(_) => Self::NaT,
        }
    }

    #[cold]
    #[cfg_attr(feature = "optimize", optimize(size))]
    pub fn datetime(self, val: i64, opts: Opt) -> Result<NumpyDatetime64Repr, NumpyDateTimeError> {
        let datetime = match self {
            Self::Years => {
                let year = val + 1970;
                if !(0..=9999).contains(&year) {
                    cold_path!();
                    return Err(NumpyDateTimeError::Unrepresentable { unit: self, val });
                } else {
                    Ok(DateTime::new(year as i16, 1, 1, 0, 0, 0, 0).unwrap())
                }
            }
            Self::Months => {
                let year = val / 12 + 1970;
                let month = val % 12 + 1;
                if !(0..=9999).contains(&year) || !(0..=12).contains(&month) {
                    cold_path!();
                    return Err(NumpyDateTimeError::Unrepresentable { unit: self, val });
                } else {
                    Ok(DateTime::new(year as i16, month as i8, 1, 0, 0, 0, 0).unwrap())
                }
            }
            Self::Weeks => {
                to_jiff_datetime!(Timestamp::from_second(val * 7 * 24 * 60 * 60), self, val)
            }
            Self::Days => to_jiff_datetime!(Timestamp::from_second(val * 24 * 60 * 60), self, val),
            Self::Hours => to_jiff_datetime!(Timestamp::from_second(val * 60 * 60), self, val),
            Self::Minutes => to_jiff_datetime!(Timestamp::from_second(val * 60), self, val),
            Self::Seconds => to_jiff_datetime!(Timestamp::from_second(val), self, val),
            Self::Milliseconds => to_jiff_datetime!(Timestamp::from_millisecond(val), self, val),
            Self::Microseconds => to_jiff_datetime!(Timestamp::from_microsecond(val), self, val),
            Self::Nanoseconds => {
                to_jiff_datetime!(Timestamp::from_nanosecond(i128::from(val)), self, val)
            }
            _ => Err(NumpyDateTimeError::UnsupportedUnit(self)),
        };
        match datetime {
            Ok(dt) => match dt.year() {
                0..=9999 => Ok(NumpyDatetime64Repr { dt, opts }),
                _ => Err(NumpyDateTimeError::Unrepresentable { unit: self, val }),
            },
            Err(err) => Err(err),
        }
    }
}

macro_rules! forward_inner {
    ($meth: ident, $ty: ident) => {
        pub fn $meth(&self) -> $ty {
            debug_assert!(self.dt.$meth() >= 0);
            #[allow(clippy::cast_sign_loss)]
            let ret = self.dt.$meth() as $ty; // stmt_expr_attributes
            ret
        }
    };
}

pub(crate) struct NumpyDatetime64Repr {
    pub dt: DateTime,
    pub opts: Opt,
}

impl NumpyDatetime64Repr {
    forward_inner!(year, i32);
    forward_inner!(month, u8);
    forward_inner!(day, u8);
    forward_inner!(hour, u8);
    forward_inner!(minute, u8);
    forward_inner!(second, u8);

    pub fn nanosecond(&self) -> u32 {
        debug_assert!(self.dt.subsec_nanosecond() >= 0);
        self.dt.subsec_nanosecond().cast_unsigned()
    }

    pub fn microsecond(&self) -> u32 {
        self.nanosecond() / 1_000
    }
}
