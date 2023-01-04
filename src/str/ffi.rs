// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ffi::c_void;
use pyo3_ffi::*;
use std::os::raw::c_char;

// see unicodeobject.h for documentation
// re: python3.12 changes, https://www.python.org/dev/peps/pep-0623/

#[repr(C)]
pub struct PyASCIIObject {
    pub ob_base: PyObject,
    pub length: Py_ssize_t,
    pub hash: Py_hash_t,
    pub state: u32,
    #[cfg(not(Py_3_12))]
    pub wstr: *mut c_char,
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub ob_base: PyASCIIObject,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    #[cfg(not(Py_3_12))]
    pub wstr_length: Py_ssize_t,
}

#[cfg(not(Py_3_12))]
const STATE_ASCII: u32 = 0b00000000000000000000000001000000;
#[cfg(not(Py_3_12))]
const STATE_COMPACT: u32 = 0b00000000000000000000000000100000;

#[cfg(Py_3_12)]
const STATE_ASCII: u32 = 0b00000000000000000000000000100000;

#[cfg(Py_3_12)]
const STATE_COMPACT: u32 = 0b00000000000000000000000000010000;

const STATE_COMPACT_ASCII: u32 = STATE_COMPACT | STATE_ASCII;

#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        let data_ptr: *mut c_void;
        if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT_ASCII == STATE_COMPACT_ASCII {
            data_ptr = (op as *mut PyASCIIObject).offset(1) as *mut c_void;
        } else {
            data_ptr = (op as *mut PyCompactUnicodeObject).offset(1) as *mut c_void;
        }
        let num_bytes = (*(op as *mut PyASCIIObject)).length
            * (((*(op as *mut PyASCIIObject)).state >> 2) & 7) as isize;
        let hash = pyo3_ffi::_Py_HashBytes(data_ptr, num_bytes);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[inline(never)]
pub fn unicode_to_str_via_ffi(op: *mut PyObject) -> Option<&'static str> {
    let mut str_size: pyo3_ffi::Py_ssize_t = 0;
    let ptr = ffi!(PyUnicode_AsUTF8AndSize(op, &mut str_size)) as *const u8;
    if unlikely!(ptr.is_null()) {
        None
    } else {
        Some(str_from_slice!(ptr, str_size as usize))
    }
}

#[inline(always)]
pub fn unicode_to_str(op: *mut PyObject) -> Option<&'static str> {
    unsafe {
        if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT_ASCII == STATE_COMPACT_ASCII {
            let ptr = op.cast::<PyASCIIObject>().offset(1) as *const u8;
            let len = (*op.cast::<PyASCIIObject>()).length as usize;
            Some(str_from_slice!(ptr, len))
        } else if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT == STATE_COMPACT
            && (*op.cast::<PyCompactUnicodeObject>()).utf8_length != 0
        {
            let ptr = (*op.cast::<PyCompactUnicodeObject>()).utf8 as *const u8;
            let len = (*op.cast::<PyCompactUnicodeObject>()).utf8_length as usize;
            Some(str_from_slice!(ptr, len))
        } else {
            unicode_to_str_via_ffi(op)
        }
    }
}
