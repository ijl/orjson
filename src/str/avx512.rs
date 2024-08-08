// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::str::pyunicode_new::*;

use core::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_cmpgt_epu8_mask, _mm256_cmpneq_epi8_mask, _mm256_loadu_si256,
    _mm256_mask_cmpneq_epi8_mask, _mm256_maskz_loadu_epi8, _mm256_max_epu8, _mm256_set1_epi8,
};

macro_rules! u8_as_i8 {
    ($val:expr) => {
        core::mem::transmute::<u8, i8>($val)
    };
}

macro_rules! impl_kind_simd_avx512vl {
    ($buf:expr) => {
        unsafe {
            const STRIDE: usize = 32;

            assume!($buf.len() > 0);

            let num_loops = $buf.len() / STRIDE;
            let remainder = $buf.len() % STRIDE;

            let remainder_mask: u32 = !(u32::MAX << remainder);
            let mut str_vec =
                _mm256_maskz_loadu_epi8(remainder_mask, $buf.as_bytes().as_ptr() as *const i8);
            let sptr = $buf.as_bytes().as_ptr().add(remainder);

            for i in 0..num_loops {
                str_vec = _mm256_max_epu8(
                    str_vec,
                    _mm256_loadu_si256(sptr.add(STRIDE * i) as *const __m256i),
                );
            }

            let vec_128 = _mm256_set1_epi8(u8_as_i8!(0b10000000));
            if _mm256_cmpgt_epu8_mask(str_vec, vec_128) == 0 {
                pyunicode_ascii($buf.as_bytes().as_ptr(), $buf.len())
            } else {
                let is_four = _mm256_cmpgt_epu8_mask(str_vec, _mm256_set1_epi8(u8_as_i8!(239))) > 0;
                let is_not_latin =
                    _mm256_cmpgt_epu8_mask(str_vec, _mm256_set1_epi8(u8_as_i8!(195))) > 0;
                let multibyte = _mm256_set1_epi8(u8_as_i8!(0b11000000));

                let mut num_chars = popcnt!(_mm256_mask_cmpneq_epi8_mask(
                    remainder_mask,
                    _mm256_and_si256(
                        _mm256_maskz_loadu_epi8(
                            remainder_mask,
                            $buf.as_bytes().as_ptr() as *const i8
                        ),
                        multibyte
                    ),
                    vec_128
                ));

                for i in 0..num_loops {
                    num_chars += popcnt!(_mm256_cmpneq_epi8_mask(
                        _mm256_and_si256(
                            _mm256_loadu_si256(sptr.add(STRIDE * i) as *const __m256i),
                            multibyte
                        ),
                        vec_128,
                    )) as usize;
                }

                if is_four {
                    pyunicode_fourbyte($buf, num_chars)
                } else if is_not_latin {
                    pyunicode_twobyte($buf, num_chars)
                } else {
                    pyunicode_onebyte($buf, num_chars)
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
pub unsafe fn create_str_impl_avx512vl(buf: &str) -> *mut pyo3_ffi::PyObject {
    impl_kind_simd_avx512vl!(buf)
}

#[inline(always)]
pub fn unicode_from_str(buf: &str) -> *mut pyo3_ffi::PyObject {
    unsafe {
        if unlikely!(buf.is_empty()) {
            return use_immortal!(crate::typeref::EMPTY_UNICODE);
        }
        if std::is_x86_feature_detected!("avx512vl") {
            create_str_impl_avx512vl(buf)
        } else {
            super::scalar::unicode_from_str(buf)
        }
    }
}
