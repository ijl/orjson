// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod escape;
#[macro_use]
mod scalar;

#[cfg(target_arch = "x86_64")]
mod sse2;

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
mod avx512;

#[cfg(feature = "unstable-simd")]
mod generic;

#[cfg(all(not(feature = "unstable-simd"), not(target_arch = "x86_64")))]
pub use scalar::format_escaped_str_scalar;

#[allow(unused_imports)]
#[cfg(feature = "unstable-simd")]
pub use generic::format_escaped_str_impl_generic_128;

#[cfg(all(feature = "unstable-simd", target_arch = "x86_64", feature = "avx512"))]
pub use avx512::format_escaped_str_impl_512vl;

#[allow(unused_imports)]
#[cfg(target_arch = "x86_64")]
pub use sse2::format_escaped_str_impl_sse2_128;
