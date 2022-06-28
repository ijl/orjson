// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3_ffi::{PyDictObject, PyObject, Py_ssize_t};

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyDict_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    (*op.cast::<PyDictObject>()).ma_used
}
