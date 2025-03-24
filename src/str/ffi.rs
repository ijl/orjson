// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::util::isize_to_usize;
use core::ffi::c_void;
use pyo3_ffi::{PyASCIIObject, PyCompactUnicodeObject, PyObject, Py_hash_t};

// see unicodeobject.h for documentation

#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        let data_ptr: *mut c_void = if (*op.cast::<PyASCIIObject>()).compact() == 1
            && (*op.cast::<PyASCIIObject>()).ascii() == 1
        {
            op.cast::<PyASCIIObject>().offset(1).cast::<c_void>()
        } else {
            op.cast::<PyCompactUnicodeObject>()
                .offset(1)
                .cast::<c_void>()
        };
        let num_bytes =
            (*op.cast::<PyASCIIObject>()).length * ((*op.cast::<PyASCIIObject>()).kind()) as isize;
        #[cfg(Py_3_14)]
        let hash = pyo3_ffi::Py_HashBuffer(data_ptr, num_bytes);
        #[cfg(not(Py_3_14))]
        let hash = pyo3_ffi::_Py_HashBytes(data_ptr, num_bytes);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[inline(never)]
pub fn unicode_to_str_via_ffi(op: *mut PyObject) -> Option<&'static str> {
    let mut str_size: pyo3_ffi::Py_ssize_t = 0;
    let ptr = ffi!(PyUnicode_AsUTF8AndSize(op, &mut str_size)).cast::<u8>();
    if unlikely!(ptr.is_null()) {
        None
    } else {
        Some(str_from_slice!(ptr, isize_to_usize(str_size)))
    }
}

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
