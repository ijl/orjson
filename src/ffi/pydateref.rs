// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::{PyDateTime_GET_DAY, PyDateTime_GET_MONTH, PyDateTime_GET_YEAR, PyObject};

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct PyDateRef {
    ptr: core::ptr::NonNull<PyObject>,
}

unsafe impl Send for PyDateRef {}
unsafe impl Sync for PyDateRef {}

impl PartialEq for PyDateRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl PyDateRef {
    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut PyObject) -> Self {
        unsafe {
            debug_assert!(!ptr.is_null());
            debug_assert!(crate::ffi::PyObject_Type(ptr) == crate::typeref::DATE_TYPE);
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

    #[inline]
    pub fn year(&self) -> u32 {
        unsafe {
            let tmp = PyDateTime_GET_YEAR(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u32;
            val
        }
    }

    #[inline]
    pub fn month(&self) -> u32 {
        unsafe {
            let tmp = PyDateTime_GET_MONTH(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u32;
            val
        }
    }

    #[inline]
    pub fn day(&self) -> u32 {
        unsafe {
            let tmp = PyDateTime_GET_DAY(self.ptr.as_ptr());
            debug_assert!(tmp >= 0);
            #[allow(clippy::cast_sign_loss)]
            let val = tmp as u32;
            val
        }
    }
}
