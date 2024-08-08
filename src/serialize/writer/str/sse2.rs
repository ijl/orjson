// SPDX-License-Identifier: Apache-2.0

use super::escape::{NEED_ESCAPED, QUOTE_TAB};

use core::mem::transmute;

use core::arch::x86_64::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    _mm_setzero_si128, _mm_storeu_si128, _mm_subs_epu8,
};

macro_rules! splat_mm128 {
    ($val:expr) => {
        _mm_set1_epi8(transmute::<u8, i8>($val))
    };
}

macro_rules! impl_format_simd_sse2_128 {
    ($dst:expr, $src:expr, $value_len:expr) => {
        let last_stride_src = $src.add($value_len).sub(STRIDE);
        let mut nb: usize = $value_len;

        assume!($value_len >= STRIDE);

        let blash = splat_mm128!(b'\\');
        let quote = splat_mm128!(b'"');
        let x20 = splat_mm128!(31);
        let v0 = _mm_setzero_si128();

        unsafe {
            while nb >= STRIDE {
                let str_vec = _mm_loadu_si128($src as *const __m128i);

                let mask = _mm_movemask_epi8(_mm_or_si128(
                    _mm_or_si128(
                        _mm_cmpeq_epi8(str_vec, blash),
                        _mm_cmpeq_epi8(str_vec, quote),
                    ),
                    _mm_cmpeq_epi8(_mm_subs_epu8(str_vec, x20), v0),
                )) as u32;

                _mm_storeu_si128($dst as *mut __m128i, str_vec);

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

            if nb > 0 {
                let mut scratch: [u8; 32] = [b'a'; 32];
                let mut str_vec = _mm_loadu_si128(last_stride_src as *const __m128i);
                _mm_storeu_si128(scratch.as_mut_ptr() as *mut __m128i, str_vec);

                let mut scratch_ptr = scratch.as_mut_ptr().add(16 - nb);
                str_vec = _mm_loadu_si128(scratch_ptr as *const __m128i);

                let mut mask = _mm_movemask_epi8(_mm_or_si128(
                    _mm_or_si128(
                        _mm_cmpeq_epi8(str_vec, blash),
                        _mm_cmpeq_epi8(str_vec, quote),
                    ),
                    _mm_cmpeq_epi8(_mm_subs_epu8(str_vec, x20), v0),
                )) as u32;

                while nb > 0 {
                    _mm_storeu_si128($dst as *mut __m128i, str_vec);

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
                        str_vec = _mm_loadu_si128(scratch_ptr as *const __m128i);
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
pub unsafe fn format_escaped_str_impl_sse2_128(
    odst: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    const STRIDE: usize = 16;

    let mut dst = odst;
    let mut src = value_ptr;

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    if value_len < STRIDE {
        impl_format_scalar!(dst, src, value_len)
    } else {
        impl_format_simd_sse2_128!(dst, src, value_len);
    }

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    dst as usize - odst as usize
}
