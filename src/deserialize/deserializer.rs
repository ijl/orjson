// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2023-2026), Eric Jolibois (2021), Aarni Koskela (2021)

use crate::deserialize::DeserializeError;
use crate::deserialize::input::read_input_to_buf;
use crate::typeref::EMPTY_UNICODE;
use core::ptr::NonNull;

pub(crate) fn deserialize(
    ptr: *mut crate::ffi::PyObject,
) -> Result<NonNull<crate::ffi::PyObject>, DeserializeError<'static>> {
    debug_assert!(ffi!(Py_REFCNT(ptr)) >= 1);
    let buffer = read_input_to_buf(ptr)?;
    debug_assert!(!buffer.is_empty());

    if buffer.len() == 2 {
        cold_path!();
        match buffer.as_bytes() {
            b"[]" => {
                return Ok(nonnull!(ffi!(PyList_New(0))));
            }
            b"{}" => {
                return Ok(nonnull!(ffi!(PyDict_New())));
            }
            b"\"\"" => {
                return Ok(nonnull!(use_immortal!(EMPTY_UNICODE)));
            }
            _ => {}
        }
    }

    crate::deserialize::backend::deserialize(buffer)
}
