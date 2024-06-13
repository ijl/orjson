// SPDX-License-Identifier: Apache-2.0
// Copyright 2023-2024 liuq19, ijl
// adapted from sonic-rs' src/util/string.rs

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
use core::mem::transmute;

#[cfg(feature = "unstable-simd")]
const STRIDE_128_BIT: usize = 16;

#[cfg(feature = "unstable-simd")]
use core::simd::cmp::{SimdPartialEq, SimdPartialOrd};

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
use core::arch::x86_64::{
    __m256i, _mm256_cmpeq_epu8_mask, _mm256_cmplt_epu8_mask, _mm256_lddqu_si256,
    _mm256_maskz_loadu_epi8, _mm256_set1_epi8, _mm256_storeu_epi8,
};

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
macro_rules! splat_mm256 {
    ($val:expr) => {
        _mm256_set1_epi8(transmute::<u8, i8>($val))
    };
}

macro_rules! write_escape {
    ($escape:expr, $dst:expr) => {
        core::ptr::copy_nonoverlapping($escape.0.as_ptr(), $dst, 8);
    };
}

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
macro_rules! impl_format_simd_avx512vl {
    ($dst:expr, $src:expr, $value_len:expr) => {
        let mut nb: usize = $value_len;

        let blash = splat_mm256!(b'\\');
        let quote = splat_mm256!(b'"');
        let x20 = splat_mm256!(32);

        unsafe {
            while nb >= STRIDE {
                let str_vec = _mm256_lddqu_si256(transmute::<*const u8, *const __m256i>($src));

                _mm256_storeu_epi8($dst as *mut i8, str_vec);

                #[allow(unused_mut)]
                let mut mask = _mm256_cmpeq_epu8_mask(str_vec, blash)
                    | _mm256_cmpeq_epu8_mask(str_vec, quote)
                    | _mm256_cmplt_epu8_mask(str_vec, x20);

                if unlikely!(mask > 0) {
                    let cn = mask.trailing_zeros() as usize; // _tzcnt_u32() not inlining

                    $src = $src.add(cn);
                    let escape = QUOTE_TAB[*($src) as usize];
                    $src = $src.add(1);

                    nb -= cn;
                    nb -= 1;

                    $dst = $dst.add(cn);
                    write_escape!(escape, $dst);
                    $dst = $dst.add(escape.1 as usize);
                } else {
                    nb -= STRIDE;
                    $dst = $dst.add(STRIDE);
                    $src = $src.add(STRIDE);
                }
            }

            while nb > 0 {
                let remainder_mask: u32 = !(u32::MAX << nb);
                let str_vec = _mm256_maskz_loadu_epi8(remainder_mask, $src as *const i8);

                _mm256_storeu_epi8($dst as *mut i8, str_vec);

                let mut mask = _mm256_cmpeq_epu8_mask(str_vec, blash)
                    | _mm256_cmpeq_epu8_mask(str_vec, quote)
                    | _mm256_cmplt_epu8_mask(str_vec, x20);

                mask &= remainder_mask;

                if unlikely!(mask > 0) {
                    let cn = mask.trailing_zeros() as usize; // _tzcnt_u32() not inlining

                    $src = $src.add(cn);
                    let escape = QUOTE_TAB[*($src) as usize];
                    $src = $src.add(1);

                    nb -= cn;
                    nb -= 1;

                    $dst = $dst.add(cn);
                    write_escape!(escape, $dst);
                    $dst = $dst.add(escape.1 as usize);
                } else {
                    $dst = $dst.add(nb);
                    break;
                }
            }
        }
    };
}

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
            $omask >>= 1;
            let escape = QUOTE_TAB[*($src) as usize];
            write_escape!(escape, $dst);
            $dst = $dst.add(escape.1 as usize);
            $src = $src.add(1);
            if likely!($omask & 1 != 1) {
                break;
            }
        }
    };
}

#[cfg(feature = "unstable-simd")]
macro_rules! impl_format_simd_generic_128 {
    ($dst:expr, $src:expr, $value_len:expr) => {
        let last_stride_src = $src.add($value_len).sub(STRIDE);
        let mut nb: usize = $value_len;

        assume!($value_len >= STRIDE_128_BIT);

        const BLASH: StrVector = StrVector::from_array([b'\\'; STRIDE]);
        const QUOTE: StrVector = StrVector::from_array([b'"'; STRIDE]);
        const X20: StrVector = StrVector::from_array([32; STRIDE]);

        unsafe {
            {
                const ROTATE: bool = false;
                while nb > STRIDE {
                    let mut v = StrVector::from_slice(core::slice::from_raw_parts($src, STRIDE));
                    let mut mask =
                        (v.simd_eq(BLASH) | v.simd_eq(QUOTE) | v.simd_lt(X20)).to_bitmask() as u32;
                    v.copy_to_slice(core::slice::from_raw_parts_mut($dst, STRIDE));

                    if unlikely!(mask > 0) {
                        let cn = mask.trailing_zeros() as usize;
                        impl_escape_unchecked!($src, $dst, nb, mask, cn, v, ROTATE);
                    } else {
                        nb -= STRIDE;
                        $dst = $dst.add(STRIDE);
                        $src = $src.add(STRIDE);
                    }
                }
            }

            if nb > 0 {
                const ROTATE: bool = true;
                let mut v =
                    StrVector::from_slice(core::slice::from_raw_parts(last_stride_src, STRIDE));
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
                    v.copy_to_slice(core::slice::from_raw_parts_mut($dst, STRIDE));

                    if unlikely!(mask > 0) {
                        let cn = mask.trailing_zeros() as usize;
                        impl_escape_unchecked!($src, $dst, nb, mask, cn, v, ROTATE);
                    } else {
                        $dst = $dst.add(nb);
                        break;
                    }
                }
            }
        }
    };
}

macro_rules! impl_format_scalar {
    ($dst:expr, $src:expr, $value_len:expr) => {
        unsafe {
            for _ in 0..$value_len {
                core::ptr::write($dst, *($src));
                $src = $src.add(1);
                $dst = $dst.add(1);
                if unlikely!(NEED_ESCAPED[*($src.sub(1)) as usize] > 0) {
                    let escape = QUOTE_TAB[*($src.sub(1)) as usize];
                    write_escape!(escape, $dst.sub(1));
                    $dst = $dst.add(escape.1 as usize - 1);
                }
            }
        }
    };
}

#[inline(never)]
#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx512f"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx512bw"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx512vl"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "bmi2"))]
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

#[inline(never)]
#[cfg(feature = "unstable-simd")]
#[cfg_attr(target_arch = "aarch64", target_feature(enable = "neon"))]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "sse2"))]
pub unsafe fn format_escaped_str_impl_128(
    odst: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    const STRIDE: usize = STRIDE_128_BIT;
    const STRIDE_SATURATION: u32 = u16::MAX as u32;
    type StrVector = core::simd::u8x16;

    let mut dst = odst;
    let mut src = value_ptr;

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    if value_len < STRIDE_128_BIT {
        impl_format_scalar!(dst, src, value_len)
    } else {
        impl_format_simd_generic_128!(dst, src, value_len);
    }

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    dst as usize - odst as usize
}

#[cfg(not(feature = "unstable-simd"))]
pub unsafe fn format_escaped_str_scalar(
    odst: *mut u8,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    let mut dst = odst;
    let mut src = value_ptr;

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    impl_format_scalar!(dst, src, value_len);

    core::ptr::write(dst, b'"');
    dst = dst.add(1);

    dst as usize - odst as usize
}

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
