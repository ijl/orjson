// SPDX-License-Identifier: (Apache-2.0 OR MIT)

// longintrepr.h, _longobject, _PyLongValue

#[cfg(Py_3_12)]
const SIGN_MASK: usize = 3;
#[cfg(Py_3_12)]
const SIGN_ZERO: usize = 1;
#[cfg(Py_3_12)]
const SIGN_POSITIVE: usize = 0;

#[cfg(Py_3_12)]
#[allow(dead_code)]
struct PyLongObject {
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub lv_tag: usize,
    pub ob_digit: u8,
}

#[cfg(Py_3_12)]
pub fn pylong_is_zero(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut PyLongObject)).lv_tag & SIGN_MASK == SIGN_ZERO }
}

#[cfg(not(Py_3_12))]
pub fn pylong_is_zero(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut pyo3_ffi::PyVarObject)).ob_size == 0 }
}

#[cfg(Py_3_12)]
pub fn pylong_is_unsigned(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut PyLongObject)).lv_tag & SIGN_MASK == SIGN_POSITIVE }
}

#[cfg(not(Py_3_12))]
pub fn pylong_is_unsigned(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut pyo3_ffi::PyVarObject)).ob_size > 0 }
}
