// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(all(target_arch = "x86_64", feature = "avx512"))]
mod avx512;
mod pystr;
mod pyunicode_new;
mod scalar;

pub(crate) use pystr::{PyStr, PyStrSubclass, set_str_create_fn};
