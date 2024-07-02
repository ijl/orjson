// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::DeserializeError;
use crate::ffi::*;
use crate::str::unicode_to_str;
use crate::typeref::{BYTEARRAY_TYPE, BYTES_TYPE, MEMORYVIEW_TYPE, STR_TYPE};
use crate::util::INVALID_STR;
use core::ffi::c_char;
use std::borrow::Cow;

#[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    if std::is_x86_feature_detected!("avx2") {
        unsafe { simdutf8::basic::imp::x86::avx2::validate_utf8(buf).is_ok() }
    } else {
        encoding_rs::Encoding::utf8_valid_up_to(buf) == buf.len()
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    simdutf8::basic::from_utf8(buf).is_ok()
}

#[cfg(target_arch = "aarch64")]
fn is_valid_utf8(buf: &[u8]) -> bool {
    simdutf8::basic::from_utf8(buf).is_ok()
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    std::str::from_utf8(buf).is_ok()
}

pub fn read_input_to_buf(
    ptr: *mut pyo3_ffi::PyObject,
) -> Result<&'static [u8], DeserializeError<'static>> {
    let obj_type_ptr = ob_type!(ptr);
    let buffer: &[u8];
    if is_type!(obj_type_ptr, BYTES_TYPE) {
        buffer = unsafe {
            core::slice::from_raw_parts(
                PyBytes_AS_STRING(ptr) as *const u8,
                PyBytes_GET_SIZE(ptr) as usize,
            )
        };
        if !is_valid_utf8(buffer) {
            return Err(DeserializeError::invalid(Cow::Borrowed(INVALID_STR)));
        }
    } else if is_type!(obj_type_ptr, STR_TYPE) {
        let uni = unicode_to_str(ptr);
        if unlikely!(uni.is_none()) {
            return Err(DeserializeError::invalid(Cow::Borrowed(INVALID_STR)));
        }
        let as_str = uni.unwrap();
        buffer = unsafe { core::slice::from_raw_parts(as_str.as_ptr(), as_str.len()) };
    } else if unlikely!(is_type!(obj_type_ptr, MEMORYVIEW_TYPE)) {
        let membuf = unsafe { PyMemoryView_GET_BUFFER(ptr) };
        if unsafe { pyo3_ffi::PyBuffer_IsContiguous(membuf, b'C' as c_char) == 0 } {
            return Err(DeserializeError::invalid(Cow::Borrowed(
                "Input type memoryview must be a C contiguous buffer",
            )));
        }
        buffer = unsafe {
            core::slice::from_raw_parts((*membuf).buf as *const u8, (*membuf).len as usize)
        };
        if !is_valid_utf8(buffer) {
            return Err(DeserializeError::invalid(Cow::Borrowed(INVALID_STR)));
        }
    } else if unlikely!(is_type!(obj_type_ptr, BYTEARRAY_TYPE)) {
        buffer = unsafe {
            core::slice::from_raw_parts(
                ffi!(PyByteArray_AsString(ptr)) as *const u8,
                ffi!(PyByteArray_Size(ptr)) as usize,
            )
        };
        if !is_valid_utf8(buffer) {
            return Err(DeserializeError::invalid(Cow::Borrowed(INVALID_STR)));
        }
    } else {
        return Err(DeserializeError::invalid(Cow::Borrowed(
            "Input must be bytes, bytearray, memoryview, or str",
        )));
    }
    if unlikely!(buffer.is_empty()) {
        Err(DeserializeError::invalid(Cow::Borrowed(
            "Input is a zero-length, empty document",
        )))
    } else {
        Ok(buffer)
    }
}
