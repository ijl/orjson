// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::EMPTY_UNICODE;
use crate::typeref::STR_HASH_FUNCTION;
use pyo3::ffi::*;
use std::os::raw::c_char;

// see unicodeobject.h for documentation
// re: python3.12 changes, https://www.python.org/dev/peps/pep-0623/

#[repr(C)]
pub struct PyASCIIObject {
    pub ob_base: PyObject,
    pub length: Py_ssize_t,
    pub hash: Py_hash_t,
    pub state: u32,
    #[cfg(not(python312))]
    pub wstr: *mut c_char,
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub ob_base: PyASCIIObject,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    #[cfg(not(python312))]
    pub wstr_length: Py_ssize_t,
}

const STATE_ASCII: u32 = 0b00000000000000000000000001000000;
#[cfg(not(python312))]
const STATE_COMPACT: u32 = 0b00000000000000000000000000100000;
#[cfg(not(python312))]
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
    } else if unlikely!(encoding_rs::mem::is_str_latin1(buf)) {
        // fails fast, no obvious effect on CJK
        PyUnicodeKind::OneByte
    } else if is_four_byte(buf) {
        PyUnicodeKind::FourByte
    } else {
        PyUnicodeKind::TwoByte
    }
}

pub fn unicode_from_str(buf: &str) -> *mut pyo3::ffi::PyObject {
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
                core::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, len);
                core::ptr::write(data_ptr.add(len), 0);
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
                    core::ptr::write(data_ptr, each as u16);
                    data_ptr = data_ptr.offset(1);
                }
                core::ptr::write(data_ptr, 0);
                ptr
            },
            PyUnicodeKind::FourByte => unsafe {
                let ptr = ffi!(PyUnicode_New(num_chars, 1114111));
                (*ptr.cast::<PyASCIIObject>()).length = num_chars;
                let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u32;
                for each in buf.chars() {
                    core::ptr::write(data_ptr, each as u32);
                    data_ptr = data_ptr.offset(1);
                }
                core::ptr::write(data_ptr, 0);
                ptr
            },
        }
    }
}

#[cfg(python312)]
pub fn read_utf8_from_str(op: *mut PyObject, str_size: &mut Py_ssize_t) -> *const u8 {
    unsafe {
        if (*op.cast::<PyASCIIObject>()).state & STATE_ASCII != 0 {
            *str_size = (*op.cast::<PyASCIIObject>()).length;
            op.cast::<PyASCIIObject>().offset(1) as *const u8
        } else if !(*op.cast::<PyCompactUnicodeObject>()).utf8.is_null() {
            *str_size = (*op.cast::<PyCompactUnicodeObject>()).utf8_length;
            (*op.cast::<PyCompactUnicodeObject>()).utf8 as *const u8
        } else {
            PyUnicode_AsUTF8AndSize(op, str_size) as *const u8
        }
    }
}

#[cfg(not(python312))]
pub fn read_utf8_from_str(op: *mut PyObject, str_size: &mut Py_ssize_t) -> *const u8 {
    unsafe {
        if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT_ASCII == STATE_COMPACT_ASCII {
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
        (*op.cast::<PyASCIIObject>()).hash = STR_HASH_FUNCTION.unwrap()(op);
        (*op.cast::<PyASCIIObject>()).hash
    }
}
