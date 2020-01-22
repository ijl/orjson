// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::STR_HASH_FUNCTION;
use pyo3::ffi::*;
use std::os::raw::c_char;

// see unicodeobject.h for documentation

#[repr(C)]
pub struct PyASCIIObject {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub length: Py_ssize_t,
    pub hash: Py_hash_t,
    pub state: u32,
    pub wstr: *mut c_char,
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub length: Py_ssize_t,
    pub hash: Py_hash_t,
    pub state: u32,
    pub wstr: *mut Py_UNICODE,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    pub wstr_length: Py_ssize_t,
}

const STATE_ASCII: u32 = 0b00000000000000000000000001000000;
const STATE_COMPACT: u32 = 0b00000000000000000000000000100000;

#[inline]
pub fn read_utf8_from_str(op: *mut PyObject, str_size: &mut Py_ssize_t) -> *const u8 {
    unsafe {
        if (*op.cast::<PyASCIIObject>()).state & STATE_ASCII == STATE_ASCII {
            *str_size = (*op.cast::<PyASCIIObject>()).length;
            op.cast::<PyASCIIObject>().offset(1) as *const u8
        } else if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT == STATE_COMPACT
            && !(*op.cast::<PyCompactUnicodeObject>()).utf8.is_null()
        {
            *str_size = (*op.cast::<PyCompactUnicodeObject>()).utf8_length;
            (*op.cast::<PyCompactUnicodeObject>()).utf8 as *const u8
        } else {
            PyUnicode_AsUTF8AndSize(op, str_size) as *const u8
        }
    }
}

#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        if (*op.cast::<PyASCIIObject>()).hash == -1 {
            (*op.cast::<PyASCIIObject>()).hash = STR_HASH_FUNCTION.unwrap()(op);
        }
        (*op.cast::<PyASCIIObject>()).hash
    }
}
