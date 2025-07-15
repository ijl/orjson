// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod buffer;
mod bytes;
pub(crate) mod compat;
mod fragment;
mod long;
#[cfg(feature = "yyjson")]
pub(crate) mod yyjson;

pub(crate) use buffer::*;
pub(crate) use bytes::*;
pub(crate) use compat::*;

pub(crate) use fragment::{orjson_fragmenttype_new, Fragment};
pub(crate) use long::pylong_is_unsigned;
#[cfg(feature = "inline_int")]
pub(crate) use long::{pylong_fits_in_i32, pylong_get_inline_value, pylong_is_zero};
