// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3_ffi::{
    PyBytesObject, PyBytes_FromStringAndSize, PyObject, PyVarObject, Py_ssize_t, _PyBytes_Resize,
};
use std::os::raw::c_char;
use std::ptr::NonNull;

const BUFFER_LENGTH: usize = 1024;

pub struct BytesWriter {
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
                PyBytes_FromStringAndSize(std::ptr::null_mut(), BUFFER_LENGTH as isize)
                    as *mut PyBytesObject
            },
        }
    }

    pub fn finish(&mut self) -> NonNull<PyObject> {
        unsafe {
            std::ptr::write(self.buffer_ptr(), 0);
            (*self.bytes.cast::<PyVarObject>()).ob_size = self.len as Py_ssize_t;
            self.resize(self.len);
            NonNull::new_unchecked(self.bytes as *mut PyObject)
        }
    }

    fn buffer_ptr(&self) -> *mut u8 {
        unsafe {
            std::mem::transmute::<*mut [c_char; 1], *mut u8>(std::ptr::addr_of_mut!(
                (*self.bytes).ob_sval
            ))
            .add(self.len)
        }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.cap = len;
        unsafe {
            _PyBytes_Resize(
                std::ptr::addr_of_mut!(self.bytes) as *mut *mut PyBytesObject as *mut *mut PyObject,
                len as isize,
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, len: usize) {
        let mut cap = self.cap;
        while len >= cap {
            if len < 262144 {
                cap *= 4;
            } else {
                cap *= 2;
            }
        }
        self.resize(cap);
    }
}

impl std::io::Write for BytesWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let _ = self.write_all(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        let to_write = buf.len();
        let end_length = self.len + to_write;
        if unlikely!(end_length >= self.cap) {
            self.grow(end_length);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr() as *const u8, self.buffer_ptr(), to_write);
        };
        self.len = end_length;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

// hack based on saethlin's research and patch in https://github.com/serde-rs/json/issues/766
pub trait WriteExt: std::io::Write {
    #[inline]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        std::ptr::null_mut()
    }

    #[inline]
    fn reserve(&mut self, len: usize) {
        let _ = len;
    }

    #[inline]
    fn set_written(&mut self, len: usize) {
        let _ = len;
    }

    #[inline]
    fn write_str(&mut self, val: &str) -> std::result::Result<(), std::io::Error> {
        let _ = val;
        Ok(())
    }

    #[inline]
    unsafe fn write_reserved_fragment(
        &mut self,
        val: &[u8],
    ) -> std::result::Result<(), std::io::Error> {
        let _ = val;
        Ok(())
    }

    #[inline]
    unsafe fn write_reserved_punctuation(
        &mut self,
        val: u8,
    ) -> std::result::Result<(), std::io::Error> {
        let _ = val;
        Ok(())
    }

    #[inline]
    unsafe fn write_reserved_indent(
        &mut self,
        len: usize,
    ) -> std::result::Result<(), std::io::Error> {
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

    #[inline(always)]
    fn set_written(&mut self, len: usize) {
        self.len += len;
    }

    fn write_str(&mut self, val: &str) -> Result<(), std::io::Error> {
        let to_write = val.len();
        let end_length = self.len + to_write + 2;
        if unlikely!(end_length >= self.cap) {
            self.grow(end_length);
        }
        unsafe {
            let ptr = self.buffer_ptr();
            std::ptr::write(ptr, b'"');
            std::ptr::copy_nonoverlapping(val.as_ptr() as *const u8, ptr.add(1), to_write);
            std::ptr::write(ptr.add(to_write + 1), b'"');
        };
        self.len = end_length;
        Ok(())
    }

    unsafe fn write_reserved_fragment(
        &mut self,
        val: &[u8],
    ) -> std::result::Result<(), std::io::Error> {
        let to_write = val.len();
        unsafe {
            std::ptr::copy_nonoverlapping(val.as_ptr() as *const u8, self.buffer_ptr(), to_write);
        };
        self.len += to_write;
        Ok(())
    }

    #[inline(always)]
    unsafe fn write_reserved_punctuation(
        &mut self,
        val: u8,
    ) -> std::result::Result<(), std::io::Error> {
        unsafe { std::ptr::write(self.buffer_ptr(), val) };
        self.len += 1;
        Ok(())
    }

    #[inline(always)]
    unsafe fn write_reserved_indent(&mut self, len: usize) -> Result<(), std::io::Error> {
        unsafe {
            std::ptr::write_bytes(self.buffer_ptr(), b' ', len);
        };
        self.len += len;
        Ok(())
    }
}
