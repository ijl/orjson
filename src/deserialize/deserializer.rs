// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::DeserializeError;
use crate::deserialize::utf8::read_input_to_buf;
use crate::typeref::EMPTY_UNICODE;
use core::ptr::NonNull;

pub(crate) struct DeserializeResult {
    pub(crate) obj: NonNull<crate::ffi::PyObject>,
    pub(crate) bytes_read: usize,
}

pub(crate) fn deserialize(
    ptr: *mut crate::ffi::PyObject,
    must_read_all: bool,
) -> Result<DeserializeResult, DeserializeError<'static>> {
    debug_assert!(ffi!(Py_REFCNT(ptr)) >= 1);
    let buffer = read_input_to_buf(ptr)?;
    debug_assert!(!buffer.is_empty());

    if buffer.len() == 2 {
        cold_path!();
        if buffer == b"[]" {
            return Ok(DeserializeResult {
                obj: nonnull!(ffi!(PyList_New(0))),
                bytes_read: 2,
            });
        } else if buffer == b"{}" {
            return Ok(DeserializeResult {
                obj: nonnull!(ffi!(PyDict_New())),
                bytes_read: 2,
            });
        } else if buffer == b"\"\"" {
            unsafe {
                return Ok(DeserializeResult {
                    obj: nonnull!(use_immortal!(EMPTY_UNICODE)),
                    bytes_read: 2,
                });
            }
        }
    }
    crate::deserialize::backend::deserialize(buffer, must_read_all)
}
