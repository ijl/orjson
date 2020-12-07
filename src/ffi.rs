// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use std::os::raw::c_char;

pub type _PyCFunctionFastWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LocalPyTypeObject {
    pub ob_refcnt: pyo3::ffi::Py_ssize_t,
    pub ob_type: *mut pyo3::ffi::PyTypeObject,
    pub ma_used: pyo3::ffi::Py_ssize_t,
    pub tp_name: *const c_char,
    // ...
}

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyDict_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    (*op.cast::<PyDictObject>()).ma_used
}

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
