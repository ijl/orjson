// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::utf8::{read_buf_to_str, read_input_to_buf};
use crate::deserialize::DeserializeError;
use crate::typeref::*;
use std::ptr::NonNull;

pub fn deserialize(
    ptr: *mut pyo3_ffi::PyObject,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    let buffer = read_input_to_buf(ptr)?;
    if unlikely!(buffer.len() == 2) {
        if buffer == b"[]" {
            return Ok(nonnull!(ffi!(PyList_New(0))));
        } else if buffer == b"{}" {
            return Ok(nonnull!(ffi!(PyDict_New())));
        } else if buffer == b"\"\"" {
            ffi!(Py_INCREF(EMPTY_UNICODE));
            unsafe { return Ok(nonnull!(EMPTY_UNICODE)) }
        }
    }

    let buffer_str = read_buf_to_str(buffer)?;

    #[cfg(feature = "yyjson")]
    {
        crate::deserialize::yyjson::deserialize_yyjson(buffer_str)
    }

    #[cfg(not(feature = "yyjson"))]
    {
        crate::deserialize::json::deserialize_json(buffer_str)
    }
}
