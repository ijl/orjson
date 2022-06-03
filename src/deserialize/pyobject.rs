// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use std::ptr::NonNull;

#[inline(always)]
pub fn parse_bool(val: bool) -> NonNull<pyo3_ffi::PyObject> {
    if val {
        ffi!(Py_INCREF(TRUE));
        nonnull!(TRUE)
    } else {
        ffi!(Py_INCREF(FALSE));
        nonnull!(FALSE)
    }
}

#[inline(always)]
pub fn parse_i64(val: i64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyLong_FromLongLong(val)))
}

#[inline(always)]
pub fn parse_u64(val: u64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyLong_FromUnsignedLongLong(val)))
}

#[inline(always)]
pub fn parse_f64(val: f64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyFloat_FromDouble(val)))
}

#[inline(always)]
pub fn parse_none() -> NonNull<pyo3_ffi::PyObject> {
    ffi!(Py_INCREF(NONE));
    nonnull!(NONE)
}
