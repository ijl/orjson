// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2022-2026)

pub(crate) mod datetime;
mod error;
mod fragment;
mod non_str;
mod num;
mod numpy;
mod obtype;
mod state;
mod uuid;
pub(crate) mod writer;

use crate::ffi::PyObject;
use crate::opt::{INDENT_2, Opt};
use core::ptr::NonNull;
pub(crate) use error::SerializeError;
use state::SerializerState;
pub(crate) use writer::{
    CompactFormatter, ContainerSerializer, IndentFormatter, set_str_formatter_fn,
};

pub(crate) fn serialize(
    ptr: *mut PyObject,
    default: Option<NonNull<PyObject>>,
    opts: Opt,
) -> Result<NonNull<PyObject>, SerializeError> {
    let state = SerializerState::new(opts);
    if opt_disabled!(opts, INDENT_2) {
        ContainerSerializer::new(CompactFormatter, state, default).write(ptr)
    } else {
        ContainerSerializer::new(IndentFormatter, state, default).write(ptr)
    }
}
