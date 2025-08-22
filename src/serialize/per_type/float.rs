// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

use crate::{opt::{Opt, STRICT_FLOAT}, serialize::error::SerializeError};

pub(crate) struct FloatSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
}

impl FloatSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject, opts: Opt) -> Self {
        FloatSerializer {
            ptr: ptr,
            opts: opts,
        }
    }
}

impl Serialize for FloatSerializer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = ffi!(PyFloat_AS_DOUBLE(self.ptr));
        
        if unlikely!(opt_enabled!(self.opts, STRICT_FLOAT))
            && (val.is_nan() || val.is_infinite())
        {
            err!(SerializeError::InvalidFloat)
        }

        serializer.serialize_f64(val)
    }
}
