// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2018-2026)

use crate::ffi::{PyDateRef, PyDateTimeRef, PyTimeRef};
use crate::opt::Opt;
use crate::serialize::datetime::{write_date, write_datetime, write_time};
use crate::serialize::error::SerializeError;
use crate::serialize::writer::SmallFixedBuffer;
use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub(crate) struct Date {
    ob: PyDateRef,
}

impl Date {
    pub fn new(ob: PyDateRef) -> Self {
        Date { ob }
    }
}
impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        write_date(self.ob.clone(), &mut buf);
        serializer.serialize_unit_struct(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub(crate) struct Time {
    ob: PyTimeRef,
    opts: Opt,
}

impl Time {
    pub fn new(ob: PyTimeRef, opts: Opt) -> Self {
        Time { ob, opts }
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        if write_time(self.ob.clone(), self.opts, &mut buf).is_err() {
            err!(SerializeError::DatetimeLibraryUnsupported)
        }
        serializer.serialize_unit_struct(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}

pub(crate) struct DateTime {
    ob: PyDateTimeRef,
    opts: Opt,
}

impl DateTime {
    pub fn new(ob: PyDateTimeRef, opts: Opt) -> Self {
        DateTime { ob, opts }
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        if write_datetime(self.ob.clone(), self.opts, &mut buf).is_err() {
            err!(SerializeError::DatetimeLibraryUnsupported)
        }
        serializer.serialize_unit_struct(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
