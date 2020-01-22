// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::datetime::HYPHEN;
use crate::typeref::*;
use smallvec::SmallVec;
use std::io::Write;
use std::os::raw::c_uchar;

#[inline(never)]
pub fn write_uuid(ptr: *mut pyo3::ffi::PyObject, buf: &mut SmallVec<[u8; 36]>) {
    let value: u128;
    {
        // test_uuid_immutable, test_uuid_int
        let py_int = ffi!(PyObject_GetAttr(ptr, INT_ATTR_STR));
        ffi!(Py_DECREF(py_int));
        let buffer: [c_uchar; 16] = [0; 16];
        unsafe {
            // test_uuid_overflow
            pyo3::ffi::_PyLong_AsByteArray(
                py_int as *mut pyo3::ffi::PyLongObject,
                buffer.as_ptr() as *const c_uchar,
                16,
                1, // little_endian
                0, // is_signed
            )
        };
        value = u128::from_le_bytes(buffer);
    }

    let mut hexadecimal: SmallVec<[u8; 32]> = SmallVec::with_capacity(32);
    write!(hexadecimal, "{:032x}", value).unwrap();

    buf.extend_from_slice(&hexadecimal[..8]);
    buf.push(HYPHEN);
    buf.extend_from_slice(&hexadecimal[8..12]);
    buf.push(HYPHEN);
    buf.extend_from_slice(&hexadecimal[12..16]);
    buf.push(HYPHEN);
    buf.extend_from_slice(&hexadecimal[16..20]);
    buf.push(HYPHEN);
    buf.extend_from_slice(&hexadecimal[20..]);
}
