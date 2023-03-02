// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::str::check::is_four_byte;
use crate::typeref::EMPTY_UNICODE;
use pyo3_ffi::*;
use std::os::raw::c_char;

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
    if buf.is_empty() {
        ffi!(Py_INCREF(EMPTY_UNICODE));
        unsafe { EMPTY_UNICODE }
    } else {
        let num_chars = bytecount::num_chars(buf.as_bytes()) as isize;
        match find_str_kind(buf, num_chars as usize) {
            PyUnicodeKind::Ascii => unsafe {
                let ptr = ffi!(PyUnicode_New(num_chars, 127));
                let data_ptr = ptr.cast::<PyASCIIObject>().offset(1) as *mut u8;
                std::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, num_chars as usize);
                std::ptr::write(data_ptr.offset(num_chars), 0);
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
