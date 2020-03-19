// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::bytes::PyBytesObject;
use pyo3::ffi::*;
use std::os::raw::c_char;

const INITIAL_BUFFER: [c_char; 1024] = [0; 1024];

pub struct BytesWriter {
    cap: usize,
    len: usize,
    bytes: *mut PyBytesObject,
}

impl BytesWriter {
    #[inline]
    pub fn new() -> Self {
        BytesWriter {
            cap: 1024,
            len: 0,
            bytes: unsafe {
                PyBytes_FromStringAndSize(INITIAL_BUFFER.as_ptr(), 1024) as *mut PyBytesObject
            },
        }
    }

    #[inline]
    pub fn finish(&mut self) -> *mut PyObject {
        unsafe {
            (*self.bytes).ob_size = self.len as isize;
            self.resize(self.len as isize);
        };
        self.bytes as *mut PyObject
    }

    #[inline]
    fn buffer_ptr(&self) -> *mut u8 {
        unsafe {
            std::mem::transmute::<&[c_char; 1], *mut u8>(
                &(*self.bytes.cast::<PyBytesObject>()).ob_sval,
            )
            .offset(self.len as isize)
        }
    }

    #[inline]
    fn resize(&mut self, len: isize) {
        unsafe {
            _PyBytes_Resize(
                &mut self.bytes as *mut *mut PyBytesObject as *mut *mut PyObject,
                len as isize,
            );
        }
    }

    #[inline]
    fn grow(&mut self, len: usize) {
        while self.cap - self.len < len {
            self.cap *= 2;
        }
        let old_ptr = self.bytes.clone();
        self.resize(self.cap as isize);
        if old_ptr != self.bytes {
            unsafe {
                #[cfg(x86_64)]
                core::arch::x86_64::_mm_prefetch(
                    self.buffer_ptr() as *const i8,
                    core::arch::x86_64::_MM_HINT_T1,
                );
                #[cfg(not(x86_64))]
                core::intrinsics::prefetch_write_data(self.buffer_ptr(), 2);
            }
        };
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
            std::ptr::copy_nonoverlapping(buf.as_ptr() as *mut u8, self.buffer_ptr(), to_write);
        };
        self.len += to_write;
        Ok(to_write)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::result::Result<(), std::io::Error> {
        self.write(buf).unwrap();
        Ok(())
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}
