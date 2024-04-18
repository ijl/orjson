// SPDX-License-Identifier: Apache-2.0
// Copyright 2023-2024 liuq19, ijl
// adapted from sonic-rs' src/util/string.rs

#[cfg(feature = "unstable-simd")]
use core::simd::cmp::{SimdPartialEq, SimdPartialOrd};

#[cfg(feature = "unstable-simd")]
macro_rules! impl_escape_unchecked {
    ($src:expr, $dst:expr, $nb:expr, $omask:expr, $cn:expr, $v:expr, $rotate:expr) => {
        if $rotate == true {
            for _ in 0..$cn {
                $v = $v.rotate_elements_left::<1>();
            }
        }
        $nb -= $cn;
        $dst = $dst.add($cn);
        $src = $src.add($cn);
        $omask >>= $cn;
        loop {
            if $rotate == true {
                $v = $v.rotate_elements_left::<1>();
            }
            $nb -= 1;
            $omask = $omask >> 1;
            let escape = QUOTE_TAB[*($src) as usize];
            core::ptr::copy_nonoverlapping(escape.0.as_ptr(), $dst, 6);
            $dst = $dst.add(escape.1 as usize);
            $src = $src.add(1);
            if likely!($omask & 1 != 1) {
                break;
            }
        }
    };
}

#[cfg(feature = "unstable-simd")]
macro_rules! impl_format_simd {
    ($odptr:expr, $value_ptr:expr, $value_len:expr) => {
        let mut dptr = $odptr;
        let dstart = $odptr;
        let mut sptr = $value_ptr;
        let mut nb: usize = $value_len;

        const BLASH: StrVector = StrVector::from_array([b'\\'; STRIDE]);
        const QUOTE: StrVector = StrVector::from_array([b'"'; STRIDE]);
        const X20: StrVector = StrVector::from_array([32; STRIDE]);

        unsafe {
            *dptr = b'"';
            dptr = dptr.add(1);

            {
                const ROTATE: bool = false;
                while nb >= STRIDE {
                    let mut v = StrVector::from_slice(core::slice::from_raw_parts(sptr, STRIDE));
                    let mut mask =
                        (v.simd_eq(BLASH) | v.simd_eq(QUOTE) | v.simd_lt(X20)).to_bitmask() as u32;
                    v.copy_to_slice(core::slice::from_raw_parts_mut(dptr, STRIDE));

                    if likely!(mask == 0) {
                        nb -= STRIDE;
                        dptr = dptr.add(STRIDE);
                        sptr = sptr.add(STRIDE);
                    } else {
                        let cn = mask.trailing_zeros() as usize;
                        impl_escape_unchecked!(sptr, dptr, nb, mask, cn, v, ROTATE);
                    }
                }
            }

            {
                const ROTATE: bool = true;
                let mut v = StrVector::from_slice(core::slice::from_raw_parts(
                    sptr.add(nb).sub(STRIDE),
                    STRIDE,
                ));
                let mut to_skip = STRIDE - nb;
                while to_skip >= 4 {
                    to_skip -= 4;
                    v = v.rotate_elements_left::<4>();
                }
                while to_skip > 0 {
                    to_skip -= 1;
                    v = v.rotate_elements_left::<1>();
                }

                let mut mask = (v.simd_eq(BLASH) | v.simd_eq(QUOTE) | v.simd_lt(X20)).to_bitmask()
                    as u32
                    & (STRIDE_SATURATION >> (32 - STRIDE - nb));

                while nb > 0 {
                    v.copy_to_slice(core::slice::from_raw_parts_mut(dptr, STRIDE));

                    if likely!(mask == 0) {
                        dptr = dptr.add(nb);
                        break;
                    } else {
                        let cn = mask.trailing_zeros() as usize;
                        impl_escape_unchecked!(sptr, dptr, nb, mask, cn, v, ROTATE);
                    }
                }
            }

            *dptr = b'"';
            dptr = dptr.add(1);
        }

        return dptr as usize - dstart as usize;
    };
}

#[inline(never)]
#[cfg(feature = "unstable-simd")]
#[cfg_attr(target_arch = "aarch64", target_feature(enable = "neon"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "sse2"))]
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

#[inline(never)]
#[cfg(not(feature = "unstable-simd"))]
pub unsafe fn format_escaped_str_scalar(
    odptr: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    unsafe {
        let mut dst = odptr;
        let mut src = value_ptr;
        let mut nb = value_len;

        *dst = b'"';
        dst = dst.add(1);

        let mut clean = 0;
        while clean <= value_len.saturating_sub(8) {
            let mut escapes = 0;
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 1)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 2)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 3)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 4)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 5)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 6)) as usize);
            escapes |= *NEED_ESCAPED.get_unchecked(*(src.add(clean + 7)) as usize);
            if unlikely!(escapes != 0) {
                break;
            }
            clean += 8;
        }
        if clean > 0 {
            core::ptr::copy_nonoverlapping(src, dst, clean);
            nb -= clean;
            src = src.add(clean);
            dst = dst.add(clean);
        }
        for _ in 0..nb {
            core::ptr::write(dst, *(src));
            src = src.add(1);
            dst = dst.add(1);
            if unlikely!(NEED_ESCAPED[*(src.sub(1)) as usize] != 0) {
                let escape = QUOTE_TAB[*(src.sub(1)) as usize];
                core::ptr::copy_nonoverlapping(escape.0.as_ptr(), dst.sub(1), 6);
                dst = dst.add(escape.1 as usize - 1);
            }
        }

        *dst = b'"';
        dst = dst.add(1);

        dst as usize - odptr as usize
    }
}

#[cfg(not(feature = "unstable-simd"))]
const NEED_ESCAPED: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const QUOTE_TAB: [([u8; 7], u8); 96] = [
    (*b"\\u0000\0", 6),
    (*b"\\u0001\0", 6),
    (*b"\\u0002\0", 6),
    (*b"\\u0003\0", 6),
    (*b"\\u0004\0", 6),
    (*b"\\u0005\0", 6),
    (*b"\\u0006\0", 6),
    (*b"\\u0007\0", 6),
    (*b"\\b\0\0\0\0\0", 2),
    (*b"\\t\0\0\0\0\0", 2),
    (*b"\\n\0\0\0\0\0", 2),
    (*b"\\u000b\0", 6),
    (*b"\\f\0\0\0\0\0", 2),
    (*b"\\r\0\0\0\0\0", 2),
    (*b"\\u000e\0", 6),
    (*b"\\u000f\0", 6),
    (*b"\\u0010\0", 6),
    (*b"\\u0011\0", 6),
    (*b"\\u0012\0", 6),
    (*b"\\u0013\0", 6),
    (*b"\\u0014\0", 6),
    (*b"\\u0015\0", 6),
    (*b"\\u0016\0", 6),
    (*b"\\u0017\0", 6),
    (*b"\\u0018\0", 6),
    (*b"\\u0019\0", 6),
    (*b"\\u001a\0", 6),
    (*b"\\u001b\0", 6),
    (*b"\\u001c\0", 6),
    (*b"\\u001d\0", 6),
    (*b"\\u001e\0", 6),
    (*b"\\u001f\0", 6),
    ([0; 7], 0),
    ([0; 7], 0),
    (*b"\\\"\0\0\0\0\0", 2),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
    (*b"\\\\\0\0\0\0\0", 2),
    ([0; 7], 0),
    ([0; 7], 0),
    ([0; 7], 0),
];
