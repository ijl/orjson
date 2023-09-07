// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// copied from PyO3 when 0.19.2 was latest

#[allow(non_snake_case)]
#[cfg(all(Py_3_12, target_pointer_width = "32"))]
pub const _Py_IMMORTAL_REFCNT: pyo3_ffi::Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        std::os::raw::c_uint::MAX as pyo3_ffi::Py_ssize_t
    } else {
        // for 32-bit systems, use the lower 30 bits (see comment in CPython's object.h)
        (std::os::raw::c_uint::MAX >> 2) as pyo3_ffi::Py_ssize_t
    }
};

#[inline(always)]
#[allow(non_snake_case)]
#[cfg(all(Py_3_12, target_pointer_width = "64"))]
pub unsafe fn _Py_IsImmortal(op: *mut pyo3_ffi::PyObject) -> std::os::raw::c_int {
    (((*op).ob_refcnt.ob_refcnt as i32) < 0) as std::os::raw::c_int
}

#[inline(always)]
#[allow(non_snake_case)]
#[cfg(all(Py_3_12, target_pointer_width = "32"))]
pub unsafe fn _Py_IsImmortal(op: *mut pyo3_ffi::PyObject) -> std::os::raw::c_int {
    ((*op).ob_refcnt.ob_refcnt == _Py_IMMORTAL_REFCNT) as std::os::raw::c_int
}
