// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::serialize::buffer::SmallFixedBuffer;
use crate::typeref::INT_ATTR_STR;
use core::ffi::c_uchar;
use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct UUID {
    ptr: *mut pyo3_ffi::PyObject,
}

impl UUID {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        UUID { ptr: ptr }
    }

    #[inline(never)]
    pub fn write_buf(&self, buf: &mut SmallFixedBuffer) {
        let value: u128;
        {
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
        unsafe {
            debug_assert!(buf.len() == 0);
            let len = uuid::Uuid::from_u128(value)
                .hyphenated()
                .encode_lower(buf.as_mut_slice())
                .len();
            buf.set_written(len);
            debug_assert!(buf.len() == len);
        }
    }
}
impl Serialize for UUID {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        self.write_buf(&mut buf);
        serializer.serialize_unit_struct(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
