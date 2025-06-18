// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(feature = "avx512")]
mod avx512;
mod pystr;
mod pyunicode_new;
mod scalar;

pub(crate) use pystr::{set_str_create_fn, PyStr, PyStrSubclass};
