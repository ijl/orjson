// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2025-2026)

#[allow(unused)]
use super::{Py_TPFLAGS_DICT_SUBCLASS, PyType_GetFlags};

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct PyDictRef {
    ptr: core::ptr::NonNull<pyo3_ffi::PyObject>,
}

unsafe impl Send for PyDictRef {}
unsafe impl Sync for PyDictRef {}

impl PartialEq for PyDictRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

#[cfg(all(Py_3_12, not(Py_GIL_DISABLED)))]
#[inline]
fn reverse_pydict_incref(key: *mut pyo3_ffi::PyObject, value: *mut pyo3_ffi::PyObject) {
    unsafe {
        if crate::ffi::_Py_IsImmortal(key) == 0 {
            (*key).ob_refcnt.ob_refcnt -= 1;
        }
        if crate::ffi::_Py_IsImmortal(value) == 0 {
            (*value).ob_refcnt.ob_refcnt -= 1;
        }
    }
}

#[cfg(Py_GIL_DISABLED)]
#[inline]
fn reverse_pydict_incref(_: *mut pyo3_ffi::PyObject, value: *mut pyo3_ffi::PyObject) {
    unsafe { crate::ffi::Py_DECREF(value) }
}

#[cfg(not(Py_3_12))]
#[inline]
fn reverse_pydict_incref(key: *mut pyo3_ffi::PyObject, value: *mut pyo3_ffi::PyObject) {
    unsafe {
        (*key).ob_refcnt -= 1;
        (*value).ob_refcnt -= 1;
    }
}

impl PyDictRef {
    #[cfg(CPython)]
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        unsafe {
            let ptr = crate::ffi::PyDict_New(crate::util::usize_to_isize(cap));
            debug_assert!(!ptr.is_null());
            Self { ptr: nonnull!(ptr) }
        }
    }

    #[cfg(not(CPython))]
    #[allow(unused)]
    #[inline]
    pub fn with_capacity(_cap: usize) -> Self {
        Self::new()
    }

    #[allow(unused)]
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let ptr = crate::ffi::PyDict_New(0);
            debug_assert!(!ptr.is_null());
            Self { ptr: nonnull!(ptr) }
        }
    }

    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut pyo3_ffi::PyObject) -> Self {
        unsafe {
            debug_assert!(!ptr.is_null());
            #[cfg(Py_3_15)]
            debug_assert!(
                crate::ffi::PyObject_Type(ptr) == crate::typeref::DICT_TYPE
                    || crate::ffi::PyObject_Type(ptr) == crate::typeref::FROZENDICT_TYPE
                    || is_subclass_by_flag!(
                        PyType_GetFlags(crate::ffi::PyObject_Type(ptr)),
                        Py_TPFLAGS_DICT_SUBCLASS
                    )
            );
            #[cfg(not(Py_3_15))]
            debug_assert!(
                crate::ffi::PyObject_Type(ptr) == crate::typeref::DICT_TYPE
                    || is_subclass_by_flag!(
                        PyType_GetFlags(crate::ffi::PyObject_Type(ptr)),
                        Py_TPFLAGS_DICT_SUBCLASS
                    )
            );
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
    pub fn len(&self) -> usize {
        unsafe { crate::util::isize_to_usize(super::Py_SIZE(self.as_ptr())) }
    }

    #[cfg(CPython)]
    #[inline]
    pub fn set(&mut self, key: crate::ffi::PyStrRef, value: *mut crate::ffi::PyObject) {
        debug_assert!(unsafe { crate::ffi::Py_REFCNT(self.as_ptr()) == 1 });
        debug_assert!(key.hash() != -1);
        #[cfg(not(Py_3_13))]
        unsafe {
            let _ = crate::ffi::_PyDict_SetItem_KnownHash(
                self.as_ptr(),
                key.as_ptr(),
                value,
                key.hash(),
            );
        }
        #[cfg(Py_3_13)]
        unsafe {
            let _ = crate::ffi::_PyDict_SetItem_KnownHash_LockHeld(
                self.as_ptr().cast::<crate::ffi::PyDictObject>(),
                key.as_ptr(),
                value,
                key.hash(),
            );
        }
        reverse_pydict_incref(key.as_ptr(), value);
    }

    #[cfg(not(CPython))]
    #[inline]
    pub fn set(&mut self, key: crate::ffi::PyStrRef, value: *mut crate::ffi::PyObject) {
        unsafe {
            let _ = crate::ffi::PyDict_SetItem(self.as_ptr(), key.as_ptr(), value);
        }
        reverse_pydict_incref(key.as_ptr(), value);
    }
}
