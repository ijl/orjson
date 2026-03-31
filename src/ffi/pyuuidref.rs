// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use super::{Py_DECREF, PyLong_AsByteArray, PyLongObject, PyObject, PyObject_GetAttr};
use crate::typeref::INT_ATTR_STR;

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct PyUuidRef {
    ptr: core::ptr::NonNull<PyObject>,
}

unsafe impl Send for PyUuidRef {}
unsafe impl Sync for PyUuidRef {}

impl PartialEq for PyUuidRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl PyUuidRef {
    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut PyObject) -> Self {
        unsafe {
            debug_assert!(!ptr.is_null());
            debug_assert!(crate::ffi::PyObject_Type(ptr) == crate::typeref::UUID_TYPE);
            Self {
                ptr: core::ptr::NonNull::new_unchecked(ptr),
            }
        }
    }

    #[inline]
    pub(crate) fn value(&self, buffer: &mut [u8; 16]) {
        unsafe {
            let py_int = PyObject_GetAttr(self.ptr.as_ptr(), INT_ATTR_STR);
            PyLong_AsByteArray(py_int.cast::<PyLongObject>(), buffer as *mut u8, 16, 0, 0);
            Py_DECREF(py_int);
        }
    }
}
