// SPDX-License-Identifier: (Apache-2.0 OR MIT)
use crate::deserialize::DeserializeError;
use crate::exc::*;
use crate::ffi::*;
use crate::typeref::*;
use crate::unicode::*;
use std::borrow::Cow;
use std::os::raw::c_char;

#[cfg(target_arch = "x86_64")]
fn is_valid_utf8(buf: &[u8]) -> bool {
    if std::is_x86_feature_detected!("sse4.2") {
        simdutf8::basic::from_utf8(buf).is_ok()
    } else {
        encoding_rs::Encoding::utf8_valid_up_to(buf) == buf.len()
    }
}

#[cfg(all(target_arch = "aarch64", feature = "unstable-simd"))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    simdutf8::basic::from_utf8(buf).is_ok()
}

#[cfg(all(target_arch = "aarch64", not(feature = "unstable-simd")))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    std::str::from_utf8(buf).is_ok()
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn is_valid_utf8(buf: &[u8]) -> bool {
    std::str::from_utf8(buf).is_ok()
}

pub fn read_input_to_buf(
    ptr: *mut pyo3_ffi::PyObject,
) -> Result<&'static [u8], DeserializeError<'static>> {
    let obj_type_ptr = ob_type!(ptr);
    let contents: &[u8];
    if is_type!(obj_type_ptr, STR_TYPE) {
        let mut str_size: pyo3_ffi::Py_ssize_t = 0;
        let uni = read_utf8_from_str(ptr, &mut str_size);
        if unlikely!(uni.is_null()) {
            return Err(DeserializeError::new(Cow::Borrowed(INVALID_STR), 0, 0, ""));
        }
        contents = unsafe { std::slice::from_raw_parts(uni, str_size as usize) };
    } else {
        let buffer: *const u8;
        let length: usize;
        if is_type!(obj_type_ptr, BYTES_TYPE) {
            buffer = unsafe { PyBytes_AS_STRING(ptr) as *const u8 };
            length = unsafe { PyBytes_GET_SIZE(ptr) as usize };
        } else if is_type!(obj_type_ptr, MEMORYVIEW_TYPE) {
            let membuf = unsafe { PyMemoryView_GET_BUFFER(ptr) };
            if unsafe { pyo3_ffi::PyBuffer_IsContiguous(membuf, b'C' as c_char) == 0 } {
                return Err(DeserializeError::new(
                    Cow::Borrowed("Input type memoryview must be a C contiguous buffer"),
                    0,
                    0,
                    "",
                ));
            }
            buffer = unsafe { (*membuf).buf as *const u8 };
            length = unsafe { (*membuf).len as usize };
        } else if is_type!(obj_type_ptr, BYTEARRAY_TYPE) {
            buffer = ffi!(PyByteArray_AsString(ptr)) as *const u8;
            length = ffi!(PyByteArray_Size(ptr)) as usize;
        } else {
            return Err(DeserializeError::new(
                Cow::Borrowed("Input must be bytes, bytearray, memoryview, or str"),
                0,
                0,
                "",
            ));
        }
        contents = unsafe { std::slice::from_raw_parts(buffer, length) };
    }
    Ok(contents)
}

pub fn read_buf_to_str(contents: &[u8]) -> Result<&str, DeserializeError> {
    if !is_valid_utf8(contents) {
        return Err(DeserializeError::new(Cow::Borrowed(INVALID_STR), 0, 0, ""));
    }
    Ok(unsafe { std::str::from_utf8_unchecked(contents) })
}
