// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod dataclass;
mod datetime;
#[macro_use]
mod datetimelike;
mod default;
mod dict;
mod error;
mod float;
mod int;
mod list;
mod numpy;
mod pyenum;
mod serializer;
mod str;
mod tuple;
mod uuid;
mod writer;

#[cfg(feature = "writeext")]
mod json;

pub use serializer::serialize;
