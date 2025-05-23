// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::util::isize_to_usize;
#[cfg(not(Py_3_14))]
use pyo3_ffi::PyCompactUnicodeObject;
use pyo3_ffi::{
    PyASCIIObject, PyObject, PyUnicode_DATA, PyUnicode_GET_LENGTH, PyUnicode_KIND, Py_hash_t,
};

// see unicodeobject.h for documentation

#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        let data_ptr = PyUnicode_DATA(op);
        let num_bytes = PyUnicode_GET_LENGTH(op) * PyUnicode_KIND(op) as isize;
        #[cfg(Py_3_14)]
        let hash = pyo3_ffi::Py_HashBuffer(data_ptr, num_bytes);
        #[cfg(not(Py_3_14))]
        let hash = pyo3_ffi::_Py_HashBytes(data_ptr, num_bytes);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[inline(never)]
fn unicode_to_str_via_ffi(op: *mut PyObject) -> Option<&'static str> {
    let mut str_size: pyo3_ffi::Py_ssize_t = 0;
    let ptr = ffi!(PyUnicode_AsUTF8AndSize(op, &mut str_size)).cast::<u8>();
    if unlikely!(ptr.is_null()) {
        None
    } else {
        Some(str_from_slice!(ptr, isize_to_usize(str_size)))
    }
}

#[cfg(not(Py_3_14))]
#[inline]
pub fn unicode_to_str(op: *mut PyObject) -> Option<&'static str> {
    unsafe {
        if unlikely!((*op.cast::<PyASCIIObject>()).compact() == 0) {
            unicode_to_str_via_ffi(op)
        } else if (*op.cast::<PyASCIIObject>()).ascii() == 1 {
            let ptr = op.cast::<PyASCIIObject>().offset(1) as *const u8;
            let len = isize_to_usize((*op.cast::<PyASCIIObject>()).length);
            Some(str_from_slice!(ptr, len))
        } else if (*op.cast::<PyCompactUnicodeObject>()).utf8_length != 0 {
            let ptr = (*op.cast::<PyCompactUnicodeObject>()).utf8 as *const u8;
            let len = isize_to_usize((*op.cast::<PyCompactUnicodeObject>()).utf8_length);
            Some(str_from_slice!(ptr, len))
        } else {
            unicode_to_str_via_ffi(op)
        }
    }
}

#[cfg(Py_3_14)]
#[inline]
pub fn unicode_to_str(op: *mut PyObject) -> Option<&'static str> {
    unicode_to_str_via_ffi(op)
}
