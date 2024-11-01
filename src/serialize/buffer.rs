// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::mem::MaybeUninit;

const BUFFER_LENGTH: usize = 64 - core::mem::size_of::<usize>();

/// For use to serialize fixed-size UUIDs and DateTime.
#[repr(align(64))]
pub struct SmallFixedBuffer {
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
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (core::ptr::addr_of_mut!(self.bytes) as *mut u8).add(self.idx),
                BUFFER_LENGTH - self.idx,
            )
        }
    }

    #[inline]
    pub unsafe fn set_written(&mut self, len: usize) {
        debug_assert!(self.idx + len < BUFFER_LENGTH);
        self.idx += len;
    }

    #[inline]
    pub fn push(&mut self, value: u8) {
        debug_assert!(self.idx + 1 < BUFFER_LENGTH);
        unsafe {
            core::ptr::write(
                (core::ptr::addr_of_mut!(self.bytes) as *mut u8).add(self.idx),
                value,
            );
            self.idx += 1;
        };
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        debug_assert!(self.idx + slice.len() < BUFFER_LENGTH);
        unsafe {
            core::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                (core::ptr::addr_of_mut!(self.bytes) as *mut u8).add(self.idx),
                slice.len(),
            );
            self.idx += slice.len();
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        core::ptr::addr_of!(self.bytes) as *const u8
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.idx
    }
}
