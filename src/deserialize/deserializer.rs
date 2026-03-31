// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2024-2026)

use super::DeserializeError;
use super::input::Utf8Buffer;
use crate::ffi::{PyDictRef, PyListRef, PyStrRef};
use core::ptr::NonNull;

#[repr(transparent)]
pub struct Deserializer {
    buffer: Utf8Buffer,
}

impl Deserializer {
    #[inline]
    pub fn from_pyobject(
        ptr: *mut crate::ffi::PyObject,
    ) -> Result<Self, DeserializeError<'static>> {
        let buffer = Utf8Buffer::from_pyobject(ptr)?;
        debug_assert!(!buffer.as_str().is_empty());
        Ok(Self { buffer: buffer })
    }

    #[inline]
    pub fn deserialize(&self) -> Result<NonNull<crate::ffi::PyObject>, DeserializeError<'static>> {
        if self.buffer.len() == 2 {
            cold_path!();
            match self.buffer.as_bytes() {
                b"[]" => return Ok(PyListRef::with_capacity(0).as_non_null_ptr()),
                b"{}" => return Ok(PyDictRef::new().as_non_null_ptr()),
                b"\"\"" => return Ok(PyStrRef::empty().as_non_null_ptr()),
                _ => {}
            }
        }
        crate::deserialize::backend::deserialize(self.buffer.as_str())
    }
}

pub(crate) fn deserialize(
    ptr: *mut crate::ffi::PyObject,
) -> Result<NonNull<crate::ffi::PyObject>, DeserializeError<'static>> {
    let deserializer = Deserializer::from_pyobject(ptr)?;
    deserializer.deserialize()
}
