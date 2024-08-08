// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use super::escape::{NEED_ESCAPED, QUOTE_TAB};
use core::simd::cmp::{SimdPartialEq, SimdPartialOrd};

macro_rules! impl_format_simd_generic_128 {
    ($dst:expr, $src:expr, $value_len:expr) => {
        let last_stride_src = $src.add($value_len).sub(STRIDE);
        let mut nb: usize = $value_len;

        assume!($value_len >= STRIDE);

        const BLASH: StrVector = StrVector::from_array([b'\\'; STRIDE]);
        const QUOTE: StrVector = StrVector::from_array([b'"'; STRIDE]);
        const X20: StrVector = StrVector::from_array([32; STRIDE]);

        unsafe {
            {
                while nb >= STRIDE {
                    let v = StrVector::from_slice(core::slice::from_raw_parts($src, STRIDE));
                    let mask =
                        (v.simd_eq(BLASH) | v.simd_eq(QUOTE) | v.simd_lt(X20)).to_bitmask() as u32;
                    v.copy_to_slice(core::slice::from_raw_parts_mut($dst, STRIDE));

                    if unlikely!(mask > 0) {
                        let cn = trailing_zeros!(mask) as usize;
                        nb -= cn;
                        $dst = $dst.add(cn);
                        $src = $src.add(cn);
                        nb -= 1;
                        let escape = QUOTE_TAB[*($src) as usize];
                        write_escape!(escape, $dst);
                        $dst = $dst.add(escape.1 as usize);
                        $src = $src.add(1);
                    } else {
                        nb -= STRIDE;
                        $dst = $dst.add(STRIDE);
                        $src = $src.add(STRIDE);
                    }
                }
            }

            if nb > 0 {
                let mut scratch: [u8; 32] = [b'a'; 32];
                let mut v =
                    StrVector::from_slice(core::slice::from_raw_parts(last_stride_src, STRIDE));
                v.copy_to_slice(core::slice::from_raw_parts_mut(
                    scratch.as_mut_ptr(),
                    STRIDE,
                ));

                let mut scratch_ptr = scratch.as_mut_ptr().add(16 - nb);
                v = StrVector::from_slice(core::slice::from_raw_parts(scratch_ptr, STRIDE));
                let mut mask =
                    (v.simd_eq(BLASH) | v.simd_eq(QUOTE) | v.simd_lt(X20)).to_bitmask() as u32;

                while nb > 0 {
                    v.copy_to_slice(core::slice::from_raw_parts_mut($dst, STRIDE));
                    if unlikely!(mask > 0) {
                        let cn = trailing_zeros!(mask) as usize;
                        nb -= cn;
                        $dst = $dst.add(cn);
                        scratch_ptr = scratch_ptr.add(cn);
                        nb -= 1;
                        mask >>= cn + 1;
                        let escape = QUOTE_TAB[*(scratch_ptr) as usize];
                        write_escape!(escape, $dst);
                        $dst = $dst.add(escape.1 as usize);
                        scratch_ptr = scratch_ptr.add(1);
                        v = StrVector::from_slice(core::slice::from_raw_parts(scratch_ptr, STRIDE));
                    } else {
                        $dst = $dst.add(nb);
                        break;
                    }
                }
            }
        }
    };
}

#[allow(dead_code)]
#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "sse2,bmi1"))]
#[cfg_attr(target_arch = "aarch64", target_feature(enable = "neon"))]
pub unsafe fn format_escaped_str_impl_generic_128(
    odst: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    const STRIDE: usize = 16;
    type StrVector = core::simd::u8x16;

    let mut dst = odst;
    let mut src = value_ptr;

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    if value_len < STRIDE {
        impl_format_scalar!(dst, src, value_len)
    } else {
        impl_format_simd_generic_128!(dst, src, value_len);
    }

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    dst as usize - odst as usize
}
