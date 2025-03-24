// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(feature = "avx512")]
mod avx512;
mod ffi;
mod pyunicode_new;
mod scalar;

#[cfg(not(feature = "avx512"))]
pub use scalar::{set_str_create_fn, unicode_from_str};

#[cfg(feature = "avx512")]
pub use avx512::{set_str_create_fn, unicode_from_str};

pub use ffi::*;
