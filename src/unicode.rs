// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::EMPTY_UNICODE;
use crate::typeref::STR_HASH_FUNCTION;
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

fn is_four_byte(buf: &str) -> bool {
    let mut ret = false;
    for &each in buf.as_bytes() {
        ret |= each >= 240;
    }
    ret
}

enum PyUnicodeKind {
    Ascii,
    OneByte,
    TwoByte,
    FourByte,
}

fn find_str_kind(buf: &str, num_chars: usize) -> PyUnicodeKind {
    if buf.len() == num_chars {
        PyUnicodeKind::Ascii
    } else if is_four_byte(buf) {
        PyUnicodeKind::FourByte
    } else if encoding_rs::mem::is_str_latin1(buf) {
        PyUnicodeKind::OneByte
    } else {
        PyUnicodeKind::TwoByte
    }
}

pub fn unicode_from_str(buf: &str) -> *mut pyo3_ffi::PyObject {
    let len = buf.len();
    if unlikely!(len == 0) {
        ffi!(Py_INCREF(EMPTY_UNICODE));
        unsafe { EMPTY_UNICODE }
    } else {
        let num_chars = bytecount::num_chars(buf.as_bytes()) as isize;
        match find_str_kind(buf, num_chars as usize) {
            PyUnicodeKind::Ascii => unsafe {
                let ptr = ffi!(PyUnicode_New(len as isize, 127));
                let data_ptr = ptr.cast::<PyASCIIObject>().offset(1) as *mut u8;
                std::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, len);
                std::ptr::write(data_ptr.add(len), 0);
                ptr
            },
            PyUnicodeKind::OneByte => unsafe {
                PyUnicode_DecodeUTF8(
                    buf.as_bytes().as_ptr() as *const c_char,
                    buf.as_bytes().len() as isize,
                    "ignore\0".as_ptr() as *const c_char,
                )
            },
            PyUnicodeKind::TwoByte => unsafe {
                let ptr = ffi!(PyUnicode_New(num_chars, 65535));
                (*ptr.cast::<PyASCIIObject>()).length = num_chars;
                let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u16;
                for each in buf.chars() {
                    std::ptr::write(data_ptr, each as u16);
                    data_ptr = data_ptr.offset(1);
                }
                std::ptr::write(data_ptr, 0);
                ptr
            },
            PyUnicodeKind::FourByte => unsafe {
                let ptr = ffi!(PyUnicode_New(num_chars, 1114111));
                (*ptr.cast::<PyASCIIObject>()).length = num_chars;
                let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u32;
                for each in buf.chars() {
                    std::ptr::write(data_ptr, each as u32);
                    data_ptr = data_ptr.offset(1);
                }
                std::ptr::write(data_ptr, 0);
                ptr
            },
        }
    }
}

#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        (*op.cast::<PyASCIIObject>()).hash = STR_HASH_FUNCTION.unwrap()(op);
        (*op.cast::<PyASCIIObject>()).hash
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
            && !(*op.cast::<PyCompactUnicodeObject>()).utf8.is_null()
        {
            let ptr = (*op.cast::<PyCompactUnicodeObject>()).utf8 as *const u8;
            let len = (*op.cast::<PyCompactUnicodeObject>()).utf8_length as usize;
            Some(str_from_slice!(ptr, len))
        } else {
            unicode_to_str_via_ffi(op)
        }
    }
}
