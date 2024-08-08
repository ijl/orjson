// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::utf8::read_input_to_buf;
use crate::deserialize::DeserializeError;
use crate::typeref::EMPTY_UNICODE;
use core::ptr::NonNull;

pub fn deserialize(
    ptr: *mut pyo3_ffi::PyObject,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    debug_assert!(ffi!(Py_REFCNT(ptr)) >= 1);
    let buffer = read_input_to_buf(ptr)?;

    if unlikely!(buffer.len() == 2) {
        if buffer == b"[]" {
            return Ok(nonnull!(ffi!(PyList_New(0))));
        } else if buffer == b"{}" {
            return Ok(nonnull!(ffi!(PyDict_New())));
        } else if buffer == b"\"\"" {
            unsafe { return Ok(nonnull!(use_immortal!(EMPTY_UNICODE))) }
        }
    }

    let buffer_str = unsafe { std::str::from_utf8_unchecked(buffer) };

    crate::deserialize::backend::deserialize(buffer_str)
}
