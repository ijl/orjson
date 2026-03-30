// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2020-2026)

use crate::ffi::{Py_intptr_t, PyObject};
use core::ffi::{c_char, c_int, c_void};

#[repr(C)]
pub(crate) struct PyCapsule {
    head: PyObject,
    pub pointer: *mut c_void,
    pub name: *const c_char,
    pub context: *mut c_void,
    pub destructor: *mut c_void, // should be typedef void (*PyCapsule_Destructor)(PyObject *);
}

// https://docs.scipy.org/doc/numpy/reference/arrays.interface.html#c.__array_struct__

pub(crate) const NPY_ARRAY_C_CONTIGUOUS: c_int = 0x1;
pub(crate) const NPY_ARRAY_NOTSWAPPED: c_int = 0x200;

#[repr(C)]
pub(crate) struct PyArrayInterface {
    pub two: c_int,
    pub nd: c_int,
    pub typekind: c_char,
    pub itemsize: c_int,
    pub flags: c_int,
    pub shape: *mut Py_intptr_t,
    pub strides: *mut Py_intptr_t,
    pub data: *mut c_void,
    pub descr: *mut PyObject,
}
