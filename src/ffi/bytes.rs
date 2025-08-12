// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ffi::c_char;
use pyo3_ffi::{PyBytesObject, PyObject, Py_ssize_t};

use super::compat::Py_SIZE;

#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    unsafe { (&raw const (*op.cast::<PyBytesObject>()).ob_sval).cast::<c_char>() }
}

#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn PyBytes_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    unsafe { Py_SIZE(op.cast::<pyo3_ffi::PyVarObject>()) }
}
