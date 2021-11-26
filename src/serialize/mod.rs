// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod dataclass;
mod datetime;
#[macro_use]
mod datetimelike;
mod default;
mod dict;
mod int;
mod list;
mod numpy;
mod serializer;
mod str;
mod tuple;
mod uuid;
mod writer;

pub use serializer::serialize;
