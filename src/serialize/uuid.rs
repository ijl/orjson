// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::PyUuidRef;
use crate::serialize::writer::{WriteExt, format_hyphenated};

#[cold]
#[inline(never)]
pub(crate) fn write_uuid<B>(ob: PyUuidRef, buf: &mut B)
where
    B: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        const UUID_LENGTH: usize = 36;
        debug_assert!(buf.remaining_mut() >= UUID_LENGTH);
        let dst: &mut [u8; UUID_LENGTH] = &mut *buf.as_mut_buffer_ptr().cast::<[u8; UUID_LENGTH]>();
        format_hyphenated(ob, dst);
        buf.advance_mut(UUID_LENGTH);
    }
}
