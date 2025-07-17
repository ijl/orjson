// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::util::usize_to_isize;
use core::ptr::NonNull;
use pyo3_ffi::{PyBytesObject, PyBytes_FromStringAndSize, PyObject, PyVarObject, _PyBytes_Resize};
use std::io::Error;

const BUFFER_LENGTH: usize = 1024;

pub(crate) struct BytesWriter {
    cap: usize,
    len: usize,
    bytes: *mut PyBytesObject,
}

impl BytesWriter {
    pub fn default() -> Self {
        BytesWriter {
            cap: BUFFER_LENGTH,
            len: 0,
            bytes: unsafe {
                PyBytes_FromStringAndSize(core::ptr::null_mut(), usize_to_isize(BUFFER_LENGTH))
                    .cast::<PyBytesObject>()
            },
        }
    }

    pub fn bytes_ptr(&mut self) -> NonNull<PyObject> {
        unsafe { NonNull::new_unchecked(self.bytes.cast::<PyObject>()) }
    }
    pub fn finish(&mut self, append: bool) -> NonNull<PyObject> {
        unsafe {
            if append {
                core::ptr::write(self.buffer_ptr(), b'\n');
                self.len += 1;
            }
            core::ptr::write(self.buffer_ptr(), 0);
            (*self.bytes.cast::<PyVarObject>()).ob_size = usize_to_isize(self.len);
            self.resize(self.len);
            self.bytes_ptr()
        }
    }

    fn buffer_ptr(&self) -> *mut u8 {
        unsafe { (&raw mut (*self.bytes).ob_sval).cast::<u8>().add(self.len) }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.cap = len;
        unsafe {
            _PyBytes_Resize(
                (&raw mut self.bytes).cast::<*mut PyObject>(),
                usize_to_isize(len),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, len: usize) {
        let mut cap = self.cap;
        while len >= cap {
            cap *= 2;
        }
        self.resize(cap);
    }
}

impl std::io::Write for BytesWriter {
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Error> {
        Ok(0)
    }

    fn write_all(&mut self, _buf: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

// hack based on saethlin's research and patch in https://github.com/serde-rs/json/issues/766
pub(crate) trait WriteExt: std::io::Write {
    #[inline]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        core::ptr::null_mut()
    }

    #[inline]
    fn reserve(&mut self, len: usize) {
        let _ = len;
    }

    #[inline]
    fn has_capacity(&mut self, _len: usize) -> bool {
        false
    }

    #[inline]
    fn set_written(&mut self, len: usize) {
        let _ = len;
    }

    #[inline]
    unsafe fn write_reserved_fragment(&mut self, val: &[u8]) -> Result<(), Error> {
        let _ = val;
        Ok(())
    }

    #[inline]
    unsafe fn write_reserved_punctuation(&mut self, val: u8) -> Result<(), Error> {
        let _ = val;
        Ok(())
    }

    #[inline]
    unsafe fn write_reserved_indent(&mut self, len: usize) -> Result<(), Error> {
        let _ = len;
        Ok(())
    }
}

impl WriteExt for &mut BytesWriter {
    #[inline(always)]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        self.buffer_ptr()
    }

    #[inline(always)]
    fn reserve(&mut self, len: usize) {
        let end_length = self.len + len;
        if unlikely!(end_length >= self.cap) {
            self.grow(end_length);
        }
    }

    #[inline]
    fn has_capacity(&mut self, len: usize) -> bool {
        self.len + len <= self.cap
    }

    #[inline(always)]
    fn set_written(&mut self, len: usize) {
        self.len += len;
    }

    unsafe fn write_reserved_fragment(&mut self, val: &[u8]) -> Result<(), Error> {
        let to_write = val.len();
        unsafe {
            core::ptr::copy_nonoverlapping(val.as_ptr(), self.buffer_ptr(), to_write);
        };
        self.len += to_write;
        Ok(())
    }

    #[inline(always)]
    unsafe fn write_reserved_punctuation(&mut self, val: u8) -> Result<(), Error> {
        unsafe {
            core::ptr::write(self.buffer_ptr(), val);
        }
        self.len += 1;
        Ok(())
    }

    #[inline(always)]
    unsafe fn write_reserved_indent(&mut self, len: usize) -> Result<(), Error> {
        unsafe {
            core::ptr::write_bytes(self.buffer_ptr(), b' ', len);
        };
        self.len += len;
        Ok(())
    }
}
