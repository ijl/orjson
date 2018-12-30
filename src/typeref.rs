// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::os::raw::c_char;
use std::sync::Once;

pub static mut NONE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut TRUE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut FALSE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut STR_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut BYTES_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut DICT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut LIST_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut TUPLE_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut NONE_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut BOOL_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut INT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut FLOAT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;

static EMTPY_STR: &str = "";

static INIT: Once = Once::new();

pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        NONE = pyo3::ffi::Py_None();
        TRUE = pyo3::ffi::Py_True();
        FALSE = pyo3::ffi::Py_False();
        STR_PTR = (*pyo3::ffi::PyUnicode_FromStringAndSize(
            EMTPY_STR.as_ptr() as *const c_char,
            EMTPY_STR.len() as pyo3::ffi::Py_ssize_t,
        ))
        .ob_type;
        BYTES_PTR = (*pyo3::ffi::PyBytes_FromStringAndSize(
            EMTPY_STR.as_ptr() as *const c_char,
            EMTPY_STR.len() as pyo3::ffi::Py_ssize_t,
        ))
        .ob_type;
        DICT_PTR = (*pyo3::ffi::PyDict_New()).ob_type;
        LIST_PTR = (*pyo3::ffi::PyList_New(0 as pyo3::ffi::Py_ssize_t)).ob_type;
        TUPLE_PTR = (*pyo3::ffi::PyTuple_New(0 as pyo3::ffi::Py_ssize_t)).ob_type;
        NONE_PTR = (*NONE).ob_type;
        BOOL_PTR = (*TRUE).ob_type;
        INT_PTR = (*pyo3::ffi::PyLong_FromLong(0)).ob_type;
        FLOAT_PTR = (*pyo3::ffi::PyFloat_FromDouble(0.0)).ob_type;
    });
}
