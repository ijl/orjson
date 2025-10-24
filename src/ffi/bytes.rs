// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{Py_ssize_t, PyObject};
use core::ffi::c_char;

#[cfg(not(PyPy))]
pub(crate) use pyo3_ffi::PyBytesObject;

#[cfg(PyPy)]
pub(crate) struct PyBytesObject {
    pub ob_base: *mut PyObject,
    // todo PyObject_VAR_HEAD
    pub ob_shash: core::ffi::c_long,
    pub ob_sstate: core::ffi::c_int,
    pub ob_sval: c_char,
}

#[cfg(CPython)]
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    unsafe { (&raw const (*op.cast::<crate::ffi::PyBytesObject>()).ob_sval).cast::<c_char>() }
}

#[cfg(CPython)]
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    unsafe { super::compat::Py_SIZE(op.cast::<crate::ffi::PyVarObject>()) }
}

#[cfg(not(CPython))]
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    unsafe { pyo3_ffi::PyByteArray_AsString(op) }
}

#[cfg(not(CPython))]
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    unsafe { pyo3_ffi::PyByteArray_Size(op) }
}
