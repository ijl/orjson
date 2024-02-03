// SPDX-License-Identifier: (Apache-2.0 OR MIT)

// longintrepr.h, _longobject, _PyLongValue

#[cfg(Py_3_12)]
const SIGN_MASK: usize = 3;
#[cfg(Py_3_12)]
const SIGN_ZERO: usize = 1;

#[cfg(Py_3_12)]
#[allow(non_upper_case_globals)]
const _PyLong_NON_SIZE_BITS: usize = 3;

#[cfg(Py_3_12)]
#[repr(C)]
struct _PyLongValue {
    pub lv_tag: usize,
    pub ob_digit: u32,
}

#[cfg(Py_3_12)]
#[repr(C)]
struct PyLongObject {
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub long_value: _PyLongValue,
}

#[cfg(Py_3_12)]
#[inline(always)]
pub fn pylong_is_zero(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut PyLongObject)).long_value.lv_tag & SIGN_MASK == SIGN_ZERO }
}

#[cfg(not(Py_3_12))]
#[inline(always)]
pub fn pylong_is_zero(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut pyo3_ffi::PyVarObject)).ob_size == 0 }
}

#[cfg(Py_3_12)]
#[inline(always)]
pub fn pylong_is_unsigned(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe {
        1 - (((*(ptr as *mut PyLongObject)).long_value.lv_tag & _PyLong_NON_SIZE_BITS) as isize) > 0
    }
}

#[cfg(not(Py_3_12))]
#[inline(always)]
pub fn pylong_is_unsigned(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut pyo3_ffi::PyVarObject)).ob_size > 0 }
}

#[cfg(Py_3_12)]
#[inline(always)]
fn pylong_is_compact(ptr: *mut pyo3_ffi::PyObject) -> bool {
    unsafe { (*(ptr as *mut PyLongObject)).long_value.lv_tag < (2 << _PyLong_NON_SIZE_BITS) }
}

#[cfg(Py_3_12)]
#[inline(always)]
pub fn pylong_value_unsigned(ptr: *mut pyo3_ffi::PyObject) -> u64 {
    if pylong_is_compact(ptr) == true {
        unsafe { (*(ptr as *mut PyLongObject)).long_value.ob_digit as u64 }
    } else {
        ffi!(PyLong_AsUnsignedLongLong(ptr))
    }
}

#[cfg(not(Py_3_12))]
#[inline(always)]
pub fn pylong_value_unsigned(ptr: *mut pyo3_ffi::PyObject) -> u64 {
    ffi!(PyLong_AsUnsignedLongLong(ptr))
}

#[cfg(not(Py_3_12))]
#[inline(always)]
pub fn pylong_value_signed(ptr: *mut pyo3_ffi::PyObject) -> i64 {
    ffi!(PyLong_AsLongLong(ptr))
}

#[cfg(Py_3_12)]
#[inline(always)]
pub fn pylong_value_signed(ptr: *mut pyo3_ffi::PyObject) -> i64 {
    if pylong_is_compact(ptr) == true {
        unsafe {
            let sign = 1 - ((*(ptr as *mut PyLongObject)).long_value.lv_tag & SIGN_MASK) as i64;
            sign * (*(ptr as *mut PyLongObject)).long_value.ob_digit as i64
        }
    } else {
        ffi!(PyLong_AsLongLong(ptr))
    }
}
