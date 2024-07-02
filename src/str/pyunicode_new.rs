// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3_ffi::{PyASCIIObject, PyCompactUnicodeObject};

#[inline(never)]
pub fn pyunicode_ascii(buf: *const u8, num_chars: usize) -> *mut pyo3_ffi::PyObject {
    unsafe {
        let ptr = ffi!(PyUnicode_New(num_chars as isize, 127));
        let data_ptr = ptr.cast::<PyASCIIObject>().offset(1) as *mut u8;
        core::ptr::copy_nonoverlapping(buf, data_ptr, num_chars);
        core::ptr::write(data_ptr.add(num_chars), 0);
        ptr
    }
}

#[cold]
#[inline(never)]
pub fn pyunicode_onebyte(buf: &str, num_chars: usize) -> *mut pyo3_ffi::PyObject {
    unsafe {
        let ptr = ffi!(PyUnicode_New(num_chars as isize, 255));
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u8;
        for each in buf.chars().fuse() {
            core::ptr::write(data_ptr, each as u8);
            data_ptr = data_ptr.offset(1);
        }
        core::ptr::write(data_ptr, 0);
        ptr
    }
}

#[inline(never)]
pub fn pyunicode_twobyte(buf: &str, num_chars: usize) -> *mut pyo3_ffi::PyObject {
    unsafe {
        let ptr = ffi!(PyUnicode_New(num_chars as isize, 65535));
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u16;
        for each in buf.chars().fuse() {
            core::ptr::write(data_ptr, each as u16);
            data_ptr = data_ptr.offset(1);
        }
        core::ptr::write(data_ptr, 0);
        ptr
    }
}

#[inline(never)]
pub fn pyunicode_fourbyte(buf: &str, num_chars: usize) -> *mut pyo3_ffi::PyObject {
    unsafe {
        let ptr = ffi!(PyUnicode_New(num_chars as isize, 1114111));
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1) as *mut u32;
        for each in buf.chars().fuse() {
            core::ptr::write(data_ptr, each as u32);
            data_ptr = data_ptr.offset(1);
        }
        core::ptr::write(data_ptr, 0);
        ptr
    }
}
