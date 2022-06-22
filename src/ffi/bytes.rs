// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3_ffi::*;
use std::os::raw::c_char;

#[repr(C)]
pub struct PyBytesObject {
    pub ob_base: PyVarObject,
    pub ob_shash: Py_hash_t,
    pub ob_sval: [c_char; 1],
}

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
