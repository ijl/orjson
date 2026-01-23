// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2025-2026)

use crate::deserialize::DeserializeError;
use crate::ffi::{PyByteArrayRef, PyBytesRef, PyMemoryViewRef, PyStrRef};
use crate::util::INVALID_STR;
use std::borrow::Cow;

#[cfg(CPython)]
const INPUT_TYPE_MESSAGE: &str = "Input must be bytes, bytearray, memoryview, or str";

#[cfg(not(CPython))]
const INPUT_TYPE_MESSAGE: &str = "Input must be bytes, bytearray, or str";

pub(crate) fn read_input_to_buf(
    ptr: *mut crate::ffi::PyObject,
) -> Result<&'static str, DeserializeError<'static>> {
    let buffer: Option<&'static str>;
    if let Ok(ob) = PyBytesRef::from_ptr(ptr) {
        buffer = ob.as_str();
    } else if let Ok(ob) = PyStrRef::from_ptr(ptr) {
        buffer = ob.as_str();
    } else if let Ok(ob) = PyByteArrayRef::from_ptr(ptr) {
        buffer = ob.as_str();
    } else if let Ok(ob) = PyMemoryViewRef::from_ptr(ptr) {
        buffer = ob.as_str();
    } else {
        return Err(DeserializeError::invalid(Cow::Borrowed(INPUT_TYPE_MESSAGE)));
    }
    match buffer {
        Some(as_str) => {
            if as_str.is_empty() {
                cold_path!();
                Err(DeserializeError::invalid(Cow::Borrowed(
                    "Input is a zero-length, empty document",
                )))
            } else {
                Ok(as_str)
            }
        }
        None => {
            cold_path!();
            Err(DeserializeError::invalid(Cow::Borrowed(INVALID_STR)))
        }
    }
}
