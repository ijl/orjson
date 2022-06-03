// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod cache;
mod deserializer;
mod error;
mod pyobject;
mod utf8;

#[cfg(not(feature = "yyjson"))]
mod json;

#[cfg(feature = "yyjson")]
mod yyjson;

pub use cache::KeyMap;
pub use cache::KEY_MAP;
pub use deserializer::deserialize;
pub use error::DeserializeError;
