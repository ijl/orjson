// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::PyBytesObject;
use core::ptr::NonNull;
use pyo3::ffi::*;
use std::os::raw::c_char;

pub struct BytesWriter {
    cap: usize,
    len: usize,
    bytes: *mut PyBytesObject,
}

impl BytesWriter {
    #[inline]
    pub fn new() -> Self {
        let buf = [0; 64];
        BytesWriter {
            cap: 64,
            len: 0,
            bytes: unsafe { PyBytes_FromStringAndSize(buf.as_ptr(), 64) as *mut PyBytesObject },
        }
    }

    pub fn finish(&mut self) -> NonNull<PyObject> {
        unsafe {
            (*self.bytes.cast::<PyVarObject>()).ob_size = self.len as Py_ssize_t;
            self.resize(self.len as isize);
            NonNull::new_unchecked(self.bytes as *mut PyObject)
        }
    }

    #[inline]
    fn buffer_ptr(&self) -> *mut u8 {
        unsafe {
            std::mem::transmute::<&[c_char; 1], *mut u8>(
                &(*self.bytes.cast::<PyBytesObject>()).ob_sval,
            )
            .add(self.len)
        }
    }

    pub fn resize(&mut self, len: isize) {
        unsafe {
            _PyBytes_Resize(
                &mut self.bytes as *mut *mut PyBytesObject as *mut *mut PyObject,
                len as isize,
            );
        }
    }

    pub fn prefetch(&self) {
        unsafe { core::intrinsics::prefetch_write_data(self.buffer_ptr(), 3) };
    }

    fn grow(&mut self, len: usize) {
        while self.cap - self.len < len {
            self.cap *= 2;
        }
        self.resize(self.cap as isize);
    }
}

impl std::io::Write for BytesWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        let to_write = buf.len();
        if unlikely!(self.len + to_write > self.cap) {
            self.grow(to_write);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr() as *const u8, self.buffer_ptr(), to_write);
        };
        self.len += to_write;
        Ok(to_write)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::result::Result<(), std::io::Error> {
        let _ = self.write(buf);
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}
