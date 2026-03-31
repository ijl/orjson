// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2021-2026)

pub(crate) mod datetime;
mod error;
mod numpy;
mod obtype;
mod per_type;
mod serializer;
mod state;
mod uuid;
pub(crate) mod writer;

pub(crate) use serializer::serialize;
pub(crate) use writer::set_str_formatter_fn;
