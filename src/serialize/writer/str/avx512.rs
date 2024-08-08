// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::mem::transmute;

use super::escape::QUOTE_TAB;

use core::arch::x86_64::{
    __m256i, _mm256_cmpeq_epu8_mask, _mm256_cmplt_epu8_mask, _mm256_load_si256, _mm256_loadu_si256,
    _mm256_maskz_loadu_epi8, _mm256_storeu_epi8,
};

#[repr(C, align(32))]
struct ConstArray {
    pub data: [u8; 32],
}

const BLASH: ConstArray = ConstArray { data: [b'\\'; 32] };
const QUOTE: ConstArray = ConstArray { data: [b'"'; 32] };
const X20: ConstArray = ConstArray { data: [32; 32] };

macro_rules! impl_format_simd_avx512vl {
    ($dst:expr, $src:expr, $value_len:expr) => {
        let mut nb: usize = $value_len;

        let blash = _mm256_load_si256(BLASH.data.as_ptr() as *const __m256i);
        let quote = _mm256_load_si256(QUOTE.data.as_ptr() as *const __m256i);
        let x20 = _mm256_load_si256(X20.data.as_ptr() as *const __m256i);

        unsafe {
            while nb >= STRIDE {
                let str_vec = _mm256_loadu_si256(transmute::<*const u8, *const __m256i>($src));

                _mm256_storeu_epi8($dst as *mut i8, str_vec);

                let mask = _mm256_cmpeq_epu8_mask(str_vec, blash)
                    | _mm256_cmpeq_epu8_mask(str_vec, quote)
                    | _mm256_cmplt_epu8_mask(str_vec, x20);

                if unlikely!(mask > 0) {
                    let cn = trailing_zeros!(mask);
                    $src = $src.add(cn);
                    $dst = $dst.add(cn);
                    nb -= cn;
                    nb -= 1;

                    let escape = QUOTE_TAB[*($src) as usize];
                    $src = $src.add(1);

                    write_escape!(escape, $dst);
                    $dst = $dst.add(escape.1 as usize);
                } else {
                    nb -= STRIDE;
                    $dst = $dst.add(STRIDE);
                    $src = $src.add(STRIDE);
                }
            }

            if nb > 0 {
                loop {
                    let remainder_mask = !(u32::MAX << nb);
                    let str_vec = _mm256_maskz_loadu_epi8(remainder_mask, $src as *const i8);

                    _mm256_storeu_epi8($dst as *mut i8, str_vec);

                    let mask = (_mm256_cmpeq_epu8_mask(str_vec, blash)
                        | _mm256_cmpeq_epu8_mask(str_vec, quote)
                        | _mm256_cmplt_epu8_mask(str_vec, x20))
                        & remainder_mask;

                    if unlikely!(mask > 0) {
                        let cn = trailing_zeros!(mask);
                        $src = $src.add(cn);
                        $dst = $dst.add(cn);
                        nb -= cn;
                        nb -= 1;

                        let escape = QUOTE_TAB[*($src) as usize];
                        $src = $src.add(1);

                        write_escape!(escape, $dst);
                        $dst = $dst.add(escape.1 as usize);
                    } else {
                        $dst = $dst.add(nb);
                        break;
                    }
                }
            }
        }
    };
}

#[inline(never)]
#[cfg_attr(
    feature = "avx512",
    target_feature(enable = "avx512f,avx512bw,avx512vl,bmi2")
)]
pub unsafe fn format_escaped_str_impl_512vl(
    odst: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    const STRIDE: usize = 32;

    let mut dst = odst;
    let mut src = value_ptr;

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    impl_format_simd_avx512vl!(dst, src, value_len);

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    dst as usize - odst as usize
}
