// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2023-2026)

use crate::serialize::writer::WriteExt;
use bytes::{BufMut, buf::UninitSlice};
use core::mem::MaybeUninit;

const BUFFER_LENGTH: usize = 64 - core::mem::size_of::<usize>();

/// For use to serialize fixed-size UUIDs and DateTime.
#[repr(align(64))]
pub(crate) struct SmallFixedBuffer {
    idx: usize,
    bytes: [MaybeUninit<u8>; BUFFER_LENGTH],
}

impl SmallFixedBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            idx: 0,
            bytes: [MaybeUninit::<u8>::uninit(); BUFFER_LENGTH],
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        (&raw const self.bytes).cast::<u8>()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.idx
    }

    #[allow(clippy::inherent_to_string)]
    #[inline]
    pub fn to_string(&self) -> String {
        String::from(str_from_slice!(self.as_ptr(), self.len()))
    }
}

unsafe impl BufMut for SmallFixedBuffer {
    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.idx += cnt;
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        UninitSlice::uninit(&mut self.bytes)
    }

    #[inline]
    fn remaining_mut(&self) -> usize {
        BUFFER_LENGTH - self.idx
    }

    #[inline]
    fn put_u8(&mut self, value: u8) {
        debug_assert!(self.remaining_mut() > 8);
        unsafe {
            core::ptr::write((&raw mut self.bytes).cast::<u8>().add(self.idx), value);
            self.advance_mut(1);
        };
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        debug_assert!(self.remaining_mut() > src.len());
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr(),
                (&raw mut self.bytes).cast::<u8>().add(self.idx),
                src.len(),
            );
            self.advance_mut(src.len());
        }
    }
}

impl WriteExt for SmallFixedBuffer {
    #[inline(always)]
    fn as_mut_buffer_ptr(&mut self) -> *mut u8 {
        unsafe { self.as_ptr().cast_mut().add(self.idx) }
    }

    fn reserve(&mut self, _len: usize) {
        unimplemented!()
    }

    fn reserve_minimum(&mut self) {
        unimplemented!()
    }

    fn put_bool(&mut self, _val: bool) {
        unimplemented!()
    }

    fn put_null(&mut self) {
        unimplemented!()
    }
}
