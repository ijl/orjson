// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2025-2026), Ben Sully (2021)

use crate::typeref::{
    CONVERT_METHOD_STR, DST_STR, NORMALIZE_METHOD_STR, UTCOFFSET_METHOD_STR, ZONEINFO_TYPE,
};

use crate::ffi::{PyObject_CallMethodNoArgs, PyObject_CallMethodOneArg, PyObject_HasAttr};

#[derive(Default)]
pub(crate) struct Offset {
    pub day: i32,
    pub second: i32,
}

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct PyDateTimeRef {
    ptr: core::ptr::NonNull<pyo3_ffi::PyObject>,
}

unsafe impl Send for PyDateTimeRef {}
unsafe impl Sync for PyDateTimeRef {}

impl PartialEq for PyDateTimeRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl PyDateTimeRef {
    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut pyo3_ffi::PyObject) -> Self {
        unsafe {
            debug_assert!(!ptr.is_null());
            debug_assert!(crate::ffi::PyObject_Type(ptr) == crate::typeref::DATETIME_TYPE);
            Self {
                ptr: core::ptr::NonNull::new_unchecked(ptr),
            }
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn as_ptr(&self) -> *mut pyo3_ffi::PyObject {
        self.ptr.as_ptr()
    }

    #[inline]
    #[allow(unused)]
    pub fn as_non_null_ptr(&self) -> core::ptr::NonNull<pyo3_ffi::PyObject> {
        self.ptr
    }

    #[inline]
    #[cfg(CPython)]
    pub fn tzinfo(&self) -> *mut crate::ffi::PyObject {
        unsafe {
            let ret = (*(self.ptr.as_ptr().cast::<crate::ffi::PyDateTime_DateTime>())).tzinfo;
            debug_assert!(!ret.is_null());
            ret
        }
    }

    #[inline]
    #[cfg(not(CPython))]
    pub fn tzinfo(&self) -> *mut crate::ffi::PyObject {
        unsafe {
            let ret = crate::ffi::PyDateTime_DATE_GET_TZINFO(self.ptr.as_ptr());
            debug_assert!(!ret.is_null());
            ret
        }
    }

    #[inline]
    #[cfg(CPython)]
    pub fn has_tz(&self) -> bool {
        unsafe { (*(self.ptr.as_ptr().cast::<crate::ffi::PyDateTime_DateTime>())).hastzinfo == 1 }
    }

    #[inline]
    #[cfg(not(CPython))]
    pub fn has_tz(&self) -> bool {
        unsafe { self.tzinfo() != crate::typeref::NONE }
    }

    #[inline]
    pub fn year(&self) -> i32 {
        unsafe { crate::ffi::PyDateTime_GET_YEAR(self.ptr.as_ptr()) }
    }

    #[inline]
    pub fn month(&self) -> u8 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_GET_MONTH(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u8;
            val
        }
    }

    #[inline]
    pub fn day(&self) -> u8 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_GET_DAY(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u8;
            val
        }
    }

    #[inline]
    pub fn hour(&self) -> u8 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_DATE_GET_HOUR(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u8;
            val
        }
    }

    #[inline]
    pub fn minute(&self) -> u8 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_DATE_GET_MINUTE(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u8;
            val
        }
    }

    #[inline]
    pub fn second(&self) -> u8 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_DATE_GET_SECOND(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u8;
            val
        }
    }

    #[inline]
    pub fn microsecond(&self) -> u32 {
        unsafe {
            let tmp = crate::ffi::PyDateTime_DATE_GET_MICROSECOND(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u32;
            val
        }
    }
    #[cfg(not(CPython))]
    #[inline]
    pub fn offset(&self) -> Option<Offset> {
        unimplemented!()
    }

    #[cfg(CPython)]
    #[inline]
    pub fn offset(&self) -> Option<Offset> {
        if !self.has_tz() {
            Some(Offset::default())
        } else {
            unsafe {
                let tzinfo = self.tzinfo();
                if core::ptr::eq(crate::ffi::PyObject_Type(tzinfo), ZONEINFO_TYPE) {
                    // zoneinfo
                    let py_offset =
                        PyObject_CallMethodOneArg(tzinfo, UTCOFFSET_METHOD_STR, self.ptr.as_ptr());
                    let offset = Offset {
                        second: crate::ffi::PyDateTime_DELTA_GET_SECONDS(py_offset),
                        day: crate::ffi::PyDateTime_DELTA_GET_DAYS(py_offset),
                    };
                    crate::ffi::Py_DECREF(py_offset);
                    Some(offset)
                } else {
                    self.slow_offset(tzinfo)
                }
            }
        }
    }

    #[cfg(CPython)]
    #[cold]
    #[inline(never)]
    fn slow_offset(&self, tzinfo: *mut crate::ffi::PyObject) -> Option<Offset> {
        unsafe {
            if PyObject_HasAttr(tzinfo, CONVERT_METHOD_STR) == 1 {
                // pendulum
                let py_offset = PyObject_CallMethodNoArgs(self.ptr.as_ptr(), UTCOFFSET_METHOD_STR);
                let offset = Offset {
                    second: crate::ffi::PyDateTime_DELTA_GET_SECONDS(py_offset),
                    day: crate::ffi::PyDateTime_DELTA_GET_DAYS(py_offset),
                };
                crate::ffi::Py_DECREF(py_offset);
                Some(offset)
            } else if PyObject_HasAttr(tzinfo, NORMALIZE_METHOD_STR) == 1 {
                // pytz
                let method_ptr =
                    PyObject_CallMethodOneArg(tzinfo, NORMALIZE_METHOD_STR, self.ptr.as_ptr());
                let py_offset = PyObject_CallMethodNoArgs(method_ptr, UTCOFFSET_METHOD_STR);
                crate::ffi::Py_DECREF(method_ptr);
                let offset = Offset {
                    second: crate::ffi::PyDateTime_DELTA_GET_SECONDS(py_offset),
                    day: crate::ffi::PyDateTime_DELTA_GET_DAYS(py_offset),
                };
                crate::ffi::Py_DECREF(py_offset);

                Some(offset)
            } else if PyObject_HasAttr(tzinfo, DST_STR) == 1 {
                // dateutil/arrow, datetime.timezone.utc
                let py_offset =
                    PyObject_CallMethodOneArg(tzinfo, UTCOFFSET_METHOD_STR, self.ptr.as_ptr());
                let offset = Offset {
                    second: crate::ffi::PyDateTime_DELTA_GET_SECONDS(py_offset),
                    day: crate::ffi::PyDateTime_DELTA_GET_DAYS(py_offset),
                };
                crate::ffi::Py_DECREF(py_offset);
                Some(offset)
            } else {
                None
            }
        }
    }
}
