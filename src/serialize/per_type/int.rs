// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::{Opt, STRICT_INTEGER};
use crate::serialize::error::SerializeError;
use serde::ser::{Serialize, Serializer};

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
    #[cfg(feature = "inline_int")]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            if crate::ffi::pylong_is_zero(self.ptr) {
                return serializer.serialize_bytes(b"0");
            }
            let is_signed = !crate::ffi::pylong_is_unsigned(self.ptr) as i32;
            if crate::ffi::pylong_fits_in_i32(self.ptr) {
                if is_signed == 0 {
                    serializer.serialize_u64(crate::ffi::pylong_get_inline_value(self.ptr) as u64)
                } else {
                    serializer.serialize_i64(crate::ffi::pylong_get_inline_value(self.ptr) as i64)
                }
            } else {
                let mut buffer: [u8; 8] = [0; 8];
                let ret = pyo3_ffi::_PyLong_AsByteArray(
                    self.ptr as *mut pyo3_ffi::PyLongObject,
                    buffer.as_mut_ptr() as *mut core::ffi::c_uchar,
                    8,
                    1,
                    is_signed,
                );
                if unlikely!(ret == -1) {
                    ffi!(PyErr_Clear());
                    err!(SerializeError::Integer64Bits)
                }
                if is_signed == 0 {
                    let val = core::mem::transmute::<[u8; 8], u64>(buffer);
                    if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER))
                        && val > STRICT_INT_MAX as u64
                    {
                        err!(SerializeError::Integer53Bits)
                    }
                    serializer.serialize_u64(val)
                } else {
                    let val = core::mem::transmute::<[u8; 8], i64>(buffer);
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

    #[inline(always)]
    #[cfg(not(feature = "inline_int"))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            if crate::ffi::pylong_is_unsigned(self.ptr) {
                let val = ffi!(PyLong_AsUnsignedLongLong(self.ptr));
                if unlikely!(val == u64::MAX) && !ffi!(PyErr_Occurred()).is_null() {
                    ffi!(PyErr_Clear());
                    err!(SerializeError::Integer64Bits)
                } else if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER))
                    && val > STRICT_INT_MAX as u64
                {
                    err!(SerializeError::Integer53Bits)
                } else {
                    serializer.serialize_u64(val)
                }
            } else {
                let val = ffi!(PyLong_AsLongLong(self.ptr));
                if unlikely!(val == -1) && !ffi!(PyErr_Occurred()).is_null() {
                    ffi!(PyErr_Clear());
                    err!(SerializeError::Integer64Bits)
                } else if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER))
                    && !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&val)
                {
                    err!(SerializeError::Integer53Bits)
                } else {
                    serializer.serialize_i64(val)
                }
            }
        }
    }
}
