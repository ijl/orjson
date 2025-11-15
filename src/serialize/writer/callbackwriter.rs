// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{Py_DECREF, Py_INCREF, PyBytes_FromStringAndSize, PyObject};
use crate::serialize::writer::writer::Writer;
use crate::typeref::NONE;
use crate::util::usize_to_isize;
use bytes::{BufMut, buf::UninitSlice};
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use std::io;

pub(crate) struct CallbackWriter {
    callback: NonNull<PyObject>,
    buffer: Vec<u8>,
    flush_threshold: usize,
    initial_buffer_size: usize,
    maximum_buffer_size: usize,
    flush_error: Option<io::Error>,
}

impl CallbackWriter {
    #[inline]
    pub fn new(
        callback: NonNull<PyObject>,
        flush_threshold: usize,
        initial_buffer_size: usize,
        maximum_buffer_size: usize,
    ) -> Self {
        let initial_buffer_size = initial_buffer_size.max(512);
        let maximum_buffer_size = maximum_buffer_size.max(initial_buffer_size * 2);
        CallbackWriter {
            callback,
            buffer: Vec::with_capacity(initial_buffer_size),
            initial_buffer_size,
            maximum_buffer_size,
            flush_threshold,
            flush_error: None,
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buffer.capacity() > self.maximum_buffer_size {
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "Buffer size has exceeded maximum limit",
            ));
        }
        if self.buffer.is_empty() {
            return Ok(());
        }

        unsafe {
            let py_bytes = PyBytes_FromStringAndSize(
                self.buffer.as_ptr().cast::<i8>(),
                usize_to_isize(self.buffer.len()),
            );
            if py_bytes.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to create PyBytes object",
                ));
            }
            self.buffer.clear();
            // If we had to grow the buffer, shrink it down
            if self.buffer.capacity() >= self.initial_buffer_size * 2 {
                self.buffer.shrink_to(self.initial_buffer_size);
            }

            Py_INCREF(self.callback.as_ptr());
            let result = crate::ffi::PyObject_CallOneArg(self.callback.as_ptr(), py_bytes);
            Py_DECREF(py_bytes);
            Py_DECREF(self.callback.as_ptr());

            if result.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Callback function raised an exception",
                ));
            } else {
                Py_DECREF(result);
            }
        }

        Ok(())
    }

    #[inline]
    fn check_flush(&mut self) {
        if self.buffer.len() >= self.flush_threshold {
            if let Some(err) = self.flush().err() {
                self.flush_error = Some(err);
            }
        }
    }
}

impl Writer for CallbackWriter {
    fn abort(&mut self) {}

    fn finish(&mut self, append_newline: bool) -> io::Result<NonNull<PyObject>> {
        if append_newline {
            self.buffer.push(b'\n');
        }
        self.flush()?;

        if let Some(err) = self.flush_error.take() {
            Err(err)
        } else {
            // Return a Python None for symmetry with other writers
            Ok(nonnull!(use_immortal!(NONE)))
        }
    }
}

unsafe impl BufMut for CallbackWriter {
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        // This is weird, but basically we've already been told
        // something has been written to the buffer, so we just need to
        // update the length.
        unsafe {
            let new_len = self.buffer.len() + cnt;
            if new_len > self.buffer.capacity() {
                panic!("advance_mut would exceed buffer capacity");
            }
            self.buffer.set_len(new_len);
        }
        self.check_flush();
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        let remaining = self.buffer.capacity() - self.buffer.len();
        if remaining == 0 {
            panic!("buffer is full, did something forget to reserve space?");
        }

        unsafe {
            let ptr = self.buffer.as_mut_ptr();
            let offset_ptr = ptr.add(self.buffer.len());
            let cap = self.buffer.capacity() - self.buffer.len();
            UninitSlice::uninit(core::slice::from_raw_parts_mut(
                offset_ptr.cast::<MaybeUninit<u8>>(),
                cap,
            ))
        }
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.buffer.extend_from_slice(src);
        self.check_flush();
    }
}

// hack based on saethlin's research and patch in https://github.com/serde-rs/json/issues/766
impl crate::serialize::writer::WriteExt for &mut CallbackWriter {
    #[inline(always)]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr().wrapping_add(self.buffer.len())
    }

    #[inline(always)]
    fn reserve(&mut self, len: usize) {
        if self.buffer.len() + len > self.buffer.capacity() {
            self.buffer.reserve(len);
        }
    }
}
