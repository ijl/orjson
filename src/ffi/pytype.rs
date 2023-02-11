// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::os::raw::c_char;

#[repr(C)]
pub struct PyTypeObject {
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub ma_used: pyo3_ffi::Py_ssize_t,
    pub tp_name: *const c_char,
    // ...
}
