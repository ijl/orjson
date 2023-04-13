// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use std::io::Write;
use std::os::raw::c_uchar;
use crate::ffi::ReleasedGIL;

pub type UUIDBuffer = arrayvec::ArrayVec<u8, 36>;

pub struct UUID<'a> {
    ptr: *mut pyo3_ffi::PyObject,
    gil: &'a ReleasedGIL,
}

impl<'a> UUID<'a> {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, gil: &'a ReleasedGIL) -> Self {
        UUID { ptr: ptr, gil: gil }
    }

    #[cfg_attr(feature = "optimize", optimize(size))]
    pub fn write_buf(&self, buf: &mut UUIDBuffer) {
        let value: u128;
        {
            let mut _guard = self.gil.gil_locked();
            // test_uuid_immutable, test_uuid_int
            let py_int = ffi!(PyObject_GetAttr(self.ptr, INT_ATTR_STR));
            ffi!(Py_DECREF(py_int));
            let buffer: [c_uchar; 16] = [0; 16];
            unsafe {
                // test_uuid_overflow
                pyo3_ffi::_PyLong_AsByteArray(
                    py_int as *mut pyo3_ffi::PyLongObject,
                    buffer.as_ptr() as *mut c_uchar,
                    16,
                    1, // little_endian
                    0, // is_signed
                )
            };
            value = u128::from_le_bytes(buffer);
        }

        let mut hexadecimal = arrayvec::ArrayVec::<u8, 32>::new();
        write!(hexadecimal, "{:032x}", value).unwrap();

        buf.try_extend_from_slice(&hexadecimal[..8]).unwrap();
        buf.push(b'-');
        buf.try_extend_from_slice(&hexadecimal[8..12]).unwrap();
        buf.push(b'-');
        buf.try_extend_from_slice(&hexadecimal[12..16]).unwrap();
        buf.push(b'-');
        buf.try_extend_from_slice(&hexadecimal[16..20]).unwrap();
        buf.push(b'-');
        buf.try_extend_from_slice(&hexadecimal[20..32]).unwrap();
    }
}
impl<'a> Serialize for UUID<'a> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = arrayvec::ArrayVec::<u8, 36>::new();
        self.write_buf(&mut buf);
        serializer.serialize_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
