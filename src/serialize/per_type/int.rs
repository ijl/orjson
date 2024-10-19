// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{pylong_fits_in_i32, pylong_get_inline_value, pylong_is_unsigned, pylong_is_zero};
use crate::opt::{Opt, STRICT_INTEGER};
use crate::serialize::error::SerializeError;
use serde::ser::{Serialize, Serializer};

use core::ffi::c_uchar;
use core::mem::transmute;

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub struct IntSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
}

impl IntSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, opts: Opt) -> Self {
        IntSerializer {
            ptr: ptr,
            opts: opts,
        }
    }
}

impl Serialize for IntSerializer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            if pylong_is_zero(self.ptr) {
                return serializer.serialize_bytes(b"0");
            }
            let is_signed = !pylong_is_unsigned(self.ptr) as i32;
            if pylong_fits_in_i32(self.ptr) {
                if is_signed == 0 {
                    serializer.serialize_u64(pylong_get_inline_value(self.ptr) as u64)
                } else {
                    serializer.serialize_i64(pylong_get_inline_value(self.ptr) as i64)
                }
            } else {
                let mut buffer: [u8; 8] = [0; 8];
                let ret = pyo3_ffi::_PyLong_AsByteArray(
                    self.ptr as *mut pyo3_ffi::PyLongObject,
                    buffer.as_mut_ptr() as *mut c_uchar,
                    8,
                    1,
                    is_signed,
                );
                if unlikely!(ret == -1) {
                    ffi!(PyErr_Clear());
                    err!(SerializeError::Integer64Bits)
                }
                if is_signed == 0 {
                    let val = transmute::<[u8; 8], u64>(buffer);
                    if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER))
                        && val > STRICT_INT_MAX as u64
                    {
                        err!(SerializeError::Integer53Bits)
                    }
                    serializer.serialize_u64(val)
                } else {
                    let val = transmute::<[u8; 8], i64>(buffer);
                    if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER))
                        && !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&val)
                    {
                        err!(SerializeError::Integer53Bits)
                    }
                    serializer.serialize_i64(val)
                }
            }
        }
    }
}
