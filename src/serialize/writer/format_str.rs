// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2024-2026)

use crate::serialize::writer::byteswriter::WriteExt;

macro_rules! reserve_str {
    ($writer:expr, $value:expr) => {
        $writer.reserve($value.len() * 8 + 32);
    };
}

#[cfg(all(
    target_arch = "x86_64",
    feature = "avx512",
    not(target_feature = "avx512vl")
))]
type StrFormatter = unsafe fn(*mut u8, *const u8, usize) -> usize;

pub(crate) fn set_str_formatter_fn() {
    unsafe {
        #[cfg(all(
            target_arch = "x86_64",
            feature = "avx512",
            not(target_feature = "avx512vl")
        ))]
        if std::is_x86_feature_detected!("avx512vl") {
            STR_FORMATTER_FN = crate::serialize::writer::str::format_escaped_str_impl_512vl;
        }
    }
}

#[cfg(all(
    target_arch = "x86_64",
    feature = "avx512",
    not(target_feature = "avx512vl")
))]
static mut STR_FORMATTER_FN: StrFormatter =
    crate::serialize::writer::str::format_escaped_str_impl_sse2_128;

#[cfg(all(
    target_arch = "x86_64",
    feature = "avx512",
    target_feature = "avx512vl"
))]
#[inline(always)]
pub(crate) fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_impl_512vl(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.advance_mut(written);
    }
}
#[cfg(all(target_arch = "x86_64", not(feature = "avx512")))]
#[inline(always)]
pub(crate) fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_impl_sse2_128(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.advance_mut(written);
    }
}

#[cfg(all(
    target_arch = "x86_64",
    feature = "avx512",
    not(target_feature = "avx512vl")
))]
#[inline(always)]
pub(crate) fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        reserve_str!(writer, value);

        let written = STR_FORMATTER_FN(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.advance_mut(written);
    }
}

#[cfg(all(
    not(target_arch = "x86_64"),
    not(feature = "avx512"),
    feature = "generic_simd"
))]
#[inline(always)]
pub(crate) fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_impl_generic_128(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.advance_mut(written);
    }
}

#[cfg(all(not(target_arch = "x86_64"), not(feature = "generic_simd")))]
#[inline(always)]
pub(crate) fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + WriteExt + bytes::BufMut,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_scalar(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.advance_mut(written);
    }
}
