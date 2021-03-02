// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use std::os::raw::{c_char, c_int};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyTypeObject {
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

#[repr(C)]
pub struct _PyManagedBufferObject {
    pub ob_base: *mut pyo3::ffi::PyObject,
    pub flags: c_int,
    pub exports: Py_ssize_t,
    pub master: *mut Py_buffer,
}

#[repr(C)]
pub struct PyMemoryViewObject {
    pub ob_base: PyVarObject,
    pub mbuf: *mut _PyManagedBufferObject,
    pub hash: Py_hash_t,
    pub flags: c_int,
    pub exports: Py_ssize_t,
    pub view: Py_buffer,
    pub weakreflist: *mut PyObject,
    pub ob_array: [Py_ssize_t; 1],
}

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyMemoryView_GET_BUFFER(op: *mut PyObject) -> *const Py_buffer {
    &(*op.cast::<PyMemoryViewObject>()).view
}
