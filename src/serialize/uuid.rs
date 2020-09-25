// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use smallvec::SmallVec;
use std::io::Write;
use std::os::raw::c_uchar;

pub type UUIDBuffer = smallvec::SmallVec<[u8; 64]>;

pub struct UUID {
    ptr: *mut pyo3::ffi::PyObject,
}

impl UUID {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        UUID { ptr: ptr }
    }
    pub fn write_buf(&self, buf: &mut UUIDBuffer) {
        let value: u128;
        {
            // test_uuid_immutable, test_uuid_int
            let py_int = ffi!(PyObject_GetAttr(self.ptr, INT_ATTR_STR));
            ffi!(Py_DECREF(py_int));
            let buffer: [c_uchar; 16] = [0; 16];
            unsafe {
                // test_uuid_overflow
                pyo3::ffi::_PyLong_AsByteArray(
                    py_int as *mut pyo3::ffi::PyLongObject,
                    buffer.as_ptr() as *mut c_uchar,
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
        buf.push(b'-');
        buf.extend_from_slice(&hexadecimal[8..12]);
        buf.push(b'-');
        buf.extend_from_slice(&hexadecimal[12..16]);
        buf.push(b'-');
        buf.extend_from_slice(&hexadecimal[16..20]);
        buf.push(b'-');
        buf.extend_from_slice(&hexadecimal[20..]);
    }
}
impl<'p> Serialize for UUID {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf: UUIDBuffer = smallvec::SmallVec::with_capacity(64);
        self.write_buf(&mut buf);
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
