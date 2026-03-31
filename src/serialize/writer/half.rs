// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright half-rs Contributors (2016-2026)
// https://github.com/VoidStarKat/half-rs

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub(crate) fn f16_to_f32(i: u16) -> f32 {
    #[cfg(target_feature = "f16c")]
    unsafe {
        f16_to_f32_x86_f16c(i)
    }
    #[cfg(not(target_feature = "f16c"))]
    unsafe {
        if std::arch::is_x86_feature_detected!("f16c") {
            f16_to_f32_x86_f16c(i)
        } else {
            cold_path!();
            f16_to_f32_fallback(i)
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(crate) fn f16_to_f32(i: u16) -> f32 {
    #[cfg(target_feature = "fp16")]
    unsafe {
        f16_to_f32_fp16(i)
    }
    #[cfg(not(target_feature = "fp16"))]
    unsafe {
        if std::arch::is_aarch64_feature_detected!("fp16") {
            f16_to_f32_fp16(i)
        } else {
            cold_path!();
            f16_to_f32_fallback(i)
        }
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub(crate) fn f16_to_f32(i: u16) -> f32 {
    f16_to_f32_fallback(i)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "f16c")]
#[inline]
unsafe fn f16_to_f32_x86_f16c(i: u16) -> f32 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128, __m128i, _mm_cvtph_ps};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128, __m128i, _mm_cvtph_ps};
    use core::mem::transmute;
    unsafe {
        let vec: __m128i = transmute::<[u16; 8], __m128i>([i, 0, 0, 0, 0, 0, 0, 0]);
        let retval: [f32; 4] = transmute::<__m128, [f32; 4]>(_mm_cvtph_ps(vec));
        retval[0]
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "fp16")]
#[inline]
unsafe fn f16_to_f32_fp16(i: u16) -> f32 {
    unsafe {
        let result: f32;
        core::arch::asm!(
        "fcvt {0:s}, {1:h}",
        out(vreg) result,
        in(vreg) i,
        options(pure, nomem, nostack, preserves_flags));
        result
    }
}

#[inline]
#[allow(unused)]
const fn f16_to_f32_fallback(i: u16) -> f32 {
    if i & 0x7FFFu16 == 0 {
        return unsafe { f32::from_bits((i as u32) << 16) };
    }
    let half_sign = (i & 0x8000u16) as u32;
    let half_exp = (i & 0x7C00u16) as u32;
    let half_man = (i & 0x03FFu16) as u32;
    if half_exp == 0x7C00u32 {
        if half_man == 0 {
            return unsafe { f32::from_bits((half_sign << 16) | 0x7F80_0000u32) };
        } else {
            return unsafe {
                f32::from_bits((half_sign << 16) | 0x7FC0_0000u32 | (half_man << 13))
            };
        }
    }
    let sign = half_sign << 16;
    let unbiased_exp = (half_exp.cast_signed() >> 10) - 15;
    if half_exp == 0 {
        let e = (half_man as u16).leading_zeros() - 6;
        let exp = (127 - 15 - e) << 23;
        let man = (half_man << (14 + e)) & 0x7F_FF_FFu32;
        return unsafe { f32::from_bits(sign | exp | man) };
    }
    let exp = (unbiased_exp + 127).cast_unsigned() << 23;
    let man = (half_man & 0x03FFu32) << 13;
    unsafe { f32::from_bits(sign | exp | man) }
}
