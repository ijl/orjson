// SPDX-License-Identifier: Apache-2.0
// Copyright 2023-2024 liuq19, ijl
// adapted from sonic-rs' src/util/string.rs

use crate::typeref::PAGE_SIZE;
use core::simd::cmp::{SimdPartialEq, SimdPartialOrd};

macro_rules! impl_escape_unchecked {
    ($src:expr, $dst:expr, $nb:expr, $omask:expr, $cn:expr) => {
        $nb -= $cn;
        $dst = $dst.add($cn);
        $src = $src.add($cn);
        $omask >>= $cn;
        loop {
            $nb -= 1;
            $omask = $omask >> 1;

            if *($src) == b'"' {
                core::ptr::copy_nonoverlapping(b"\\\"".as_ptr(), $dst, 2);
                $dst = $dst.add(2);
            } else if *($src) == b'\\' {
                core::ptr::copy_nonoverlapping(b"\\\\".as_ptr(), $dst, 2);
                $dst = $dst.add(2);
            } else {
                $dst = write_unusual_escape($src, $dst);
            };

            $src = $src.add(1);
            if likely!($omask & 1 != 1) {
                break;
            }
        }
    };
}

macro_rules! impl_format_simd {
    ($odptr:expr, $value_ptr:expr, $value_len:expr) => {
        let mut dptr = $odptr;
        let dstart = $odptr;
        let mut sptr = $value_ptr;
        let mut nb: usize = $value_len;

        let blash = StrVector::from_array([b'\\'; STRIDE]);
        let quote = StrVector::from_array([b'"'; STRIDE]);
        let x20 = StrVector::from_array([32; STRIDE]);

        unsafe {
            *dptr = b'"';
            dptr = dptr.add(1);

            while nb >= STRIDE {
                let v = StrVector::from_slice(core::slice::from_raw_parts(sptr, STRIDE));
                v.copy_to_slice(core::slice::from_raw_parts_mut(dptr, STRIDE));
                let mut mask =
                    (v.simd_eq(blash) | v.simd_eq(quote) | v.simd_lt(x20)).to_bitmask() as u32;

                if likely!(mask == 0) {
                    nb -= STRIDE;
                    dptr = dptr.add(STRIDE);
                    sptr = sptr.add(STRIDE);
                } else {
                    let cn = mask.trailing_zeros() as usize;
                    impl_escape_unchecked!(sptr, dptr, nb, mask, cn);
                }
            }

            let mut v = if unlikely!(is_cross_page!(sptr)) {
                let mut v = StrVector::default();
                v.as_mut_array()[..nb].copy_from_slice(core::slice::from_raw_parts(sptr, nb));
                v
            } else {
                StrVector::from_slice(core::slice::from_raw_parts(sptr, STRIDE))
            };
            while nb > 0 {
                v.copy_to_slice(core::slice::from_raw_parts_mut(dptr, STRIDE));
                let mut mask = (v.simd_eq(blash) | v.simd_eq(quote) | v.simd_lt(x20)).to_bitmask()
                    as u32
                    & (STRIDE_SATURATION >> (32 - STRIDE - nb));

                if likely!(mask == 0) {
                    dptr = dptr.add(nb);
                    break;
                } else {
                    let cn = mask.trailing_zeros() as usize;
                    let nb_start = nb;
                    impl_escape_unchecked!(sptr, dptr, nb, mask, cn);
                    let mut consumed = nb_start - nb;
                    while consumed != 0 {
                        v = v.rotate_elements_left::<1>();
                        consumed -= 1;
                    }
                }
            }

            *dptr = b'"';
            dptr = dptr.add(1);
        }

        return dptr as usize - dstart as usize;
    };
}

macro_rules! is_cross_page {
    ($src:expr) => {
        unsafe { (($src as usize & (PAGE_SIZE - 1)) + STRIDE) > PAGE_SIZE }
    };
}

#[cold]
#[inline(never)]
fn write_unusual_escape(sptr: *const u8, dptr: *mut u8) -> *mut u8 {
    unsafe {
        debug_assert!(*sptr < 32);
        let replacement = match *(sptr) {
            0 => (*b"\\u0000\0\0", 6),
            1 => (*b"\\u0001\0\0", 6),
            2 => (*b"\\u0002\0\0", 6),
            3 => (*b"\\u0003\0\0", 6),
            4 => (*b"\\u0004\0\0", 6),
            5 => (*b"\\u0005\0\0", 6),
            6 => (*b"\\u0006\0\0", 6),
            7 => (*b"\\u0007\0\0", 6),
            8 => (*b"\\b\0\0\0\0\0\0", 2),
            9 => (*b"\\t\0\0\0\0\0\0", 2),
            10 => (*b"\\n\0\0\0\0\0\0", 2),
            11 => (*b"\\u000b\0\0", 6),
            12 => (*b"\\f\0\0\0\0\0\0", 2),
            13 => (*b"\\r\0\0\0\0\0\0", 2),
            14 => (*b"\\u000e\0\0", 6),
            15 => (*b"\\u000f\0\0", 6),
            16 => (*b"\\u0010\0\0", 6),
            17 => (*b"\\u0011\0\0", 6),
            18 => (*b"\\u0012\0\0", 6),
            19 => (*b"\\u0013\0\0", 6),
            20 => (*b"\\u0014\0\0", 6),
            21 => (*b"\\u0015\0\0", 6),
            22 => (*b"\\u0016\0\0", 6),
            23 => (*b"\\u0017\0\0", 6),
            24 => (*b"\\u0018\0\0", 6),
            25 => (*b"\\u0019\0\0", 6),
            26 => (*b"\\u001a\0\0", 6),
            27 => (*b"\\u001b\0\0", 6),
            28 => (*b"\\u001c\0\0", 6),
            29 => (*b"\\u001d\0\0", 6),
            30 => (*b"\\u001e\0\0", 6),
            31 => (*b"\\u001f\0\0", 6),
            _ => unreachable!(),
        };
        core::ptr::copy_nonoverlapping(replacement.0.as_ptr(), dptr, 8);
        dptr.add(replacement.1 as usize)
    }
}

#[inline(never)]
pub unsafe fn format_escaped_str_impl_128(
    odptr: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    const STRIDE: usize = 16;
    const STRIDE_SATURATION: u32 = u16::MAX as u32;
    type StrVector = core::simd::u8x16;

    impl_format_simd!(odptr, value_ptr, value_len);
}
