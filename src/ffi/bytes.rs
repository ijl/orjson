// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ffi::c_char;
use pyo3_ffi::{PyBytesObject, PyObject, PyVarObject, Py_ssize_t};

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    &(*op.cast::<PyBytesObject>()).ob_sval as *const c_char
}

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyBytes_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    (*op.cast::<PyVarObject>()).ob_size
}
