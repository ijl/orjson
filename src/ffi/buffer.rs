// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ffi::c_int;
use pyo3_ffi::*;

#[repr(C)]
pub struct _PyManagedBufferObject {
    pub ob_base: *mut pyo3_ffi::PyObject,
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
