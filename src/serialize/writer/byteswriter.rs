// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::util::usize_to_isize;
use bytes::{buf::UninitSlice, BufMut};
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use pyo3_ffi::{PyBytesObject, PyBytes_FromStringAndSize, PyObject, PyVarObject, _PyBytes_Resize};

const BUFFER_LENGTH: usize = 1024;

pub(crate) struct BytesWriter {
    cap: usize,
    len: usize,
    bytes: *mut PyBytesObject,
}

impl BytesWriter {
    #[inline]
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

    #[inline]
    pub fn bytes_ptr(&mut self) -> NonNull<PyObject> {
        unsafe { NonNull::new_unchecked(self.bytes.cast::<PyObject>()) }
    }

    #[inline]
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

    #[inline]
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

unsafe impl BufMut for BytesWriter {
    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.len += cnt;
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        unsafe {
            UninitSlice::uninit(core::slice::from_raw_parts_mut(
                self.buffer_ptr().cast::<MaybeUninit<u8>>(),
                self.remaining_mut(),
            ))
        }
    }

    #[inline]
    fn remaining_mut(&self) -> usize {
        self.cap - self.len
    }

    #[inline]
    fn put_u8(&mut self, value: u8) {
        debug_assert!(self.remaining_mut() > 1);
        unsafe {
            core::ptr::write(self.buffer_ptr(), value);
            self.advance_mut(1);
        }
    }

    #[inline]
    fn put_bytes(&mut self, val: u8, cnt: usize) {
        debug_assert!(self.remaining_mut() > cnt);
        unsafe {
            core::ptr::write_bytes(self.buffer_ptr(), val, cnt);
            self.advance_mut(cnt);
        };
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        debug_assert!(self.remaining_mut() > src.len());
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), self.buffer_ptr(), src.len());
            self.advance_mut(src.len());
        }
    }
}

// hack based on saethlin's research and patch in https://github.com/serde-rs/json/issues/766
pub(crate) trait WriteExt {
    #[inline]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        core::ptr::null_mut()
    }

    #[inline]
    fn reserve(&mut self, len: usize) {
        let _ = len;
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
}
