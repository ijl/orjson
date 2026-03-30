// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::{
    PyDateTime_TIME_GET_HOUR, PyDateTime_TIME_GET_MICROSECOND, PyDateTime_TIME_GET_MINUTE,
    PyDateTime_TIME_GET_SECOND, PyDateTime_Time, PyObject,
};

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct PyTimeRef {
    ptr: core::ptr::NonNull<PyObject>,
}

unsafe impl Send for PyTimeRef {}
unsafe impl Sync for PyTimeRef {}

impl PartialEq for PyTimeRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl PyTimeRef {
    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut PyObject) -> Self {
        unsafe {
            debug_assert!(!ptr.is_null());
            debug_assert!(crate::ffi::PyObject_Type(ptr) == crate::typeref::TIME_TYPE);
            Self {
                ptr: core::ptr::NonNull::new_unchecked(ptr),
            }
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn as_ptr(&self) -> *mut PyObject {
        self.ptr.as_ptr()
    }

    #[inline]
    #[allow(unused)]
    pub fn as_non_null_ptr(&self) -> core::ptr::NonNull<PyObject> {
        self.ptr
    }

    #[cfg(CPython)]
    #[inline]
    pub fn has_tz(&self) -> bool {
        unsafe { (*self.ptr.as_ptr().cast::<PyDateTime_Time>()).hastzinfo == 1 }
    }

    #[cfg(not(CPython))]
    #[inline]
    pub fn has_tz(&self) -> bool {
        unimplemented!()
    }

    #[inline]
    pub fn hour(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_HOUR(self.ptr.as_ptr()).cast_unsigned() as u8 }
    }

    #[inline]
    pub fn minute(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_MINUTE(self.ptr.as_ptr()).cast_unsigned() as u8 }
    }

    #[inline]
    pub fn second(&self) -> u8 {
        unsafe { PyDateTime_TIME_GET_SECOND(self.ptr.as_ptr()).cast_unsigned() as u8 }
    }

    #[inline]
    pub fn microsecond(&self) -> u32 {
        unsafe { PyDateTime_TIME_GET_MICROSECOND(self.ptr.as_ptr()).cast_unsigned() }
    }
}
