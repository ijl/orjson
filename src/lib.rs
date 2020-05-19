// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(core_intrinsics)]
#![feature(const_fn)]
#![allow(unused_unsafe)]

#[macro_use]
mod util;

mod array;
mod bytes;
mod dataclass;
mod datetime;
mod decode;
mod default;
mod dict;
mod encode;
mod exc;
mod ffi;
mod iter;
mod module;
mod opt;
mod typeref;
mod unicode;
mod uuid;
mod writer;

pub use module::dumps;
pub use module::loads;
pub use module::PyInit_orjson;
