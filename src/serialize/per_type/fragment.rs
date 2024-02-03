// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{Fragment, PyBytes_AS_STRING, PyBytes_GET_SIZE};
use crate::serialize::error::SerializeError;
use crate::str::unicode_to_str;
use crate::typeref::{BYTES_TYPE, STR_TYPE};

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct FragmentSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl FragmentSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        FragmentSerializer { ptr: ptr }
    }
}

impl Serialize for FragmentSerializer {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let buffer: &[u8];
        unsafe {
            let fragment: *mut Fragment = self.ptr as *mut Fragment;
            let ob_type = ob_type!((*fragment).contents);
            if ob_type == BYTES_TYPE {
                buffer = core::slice::from_raw_parts(
                    PyBytes_AS_STRING((*fragment).contents) as *const u8,
                    PyBytes_GET_SIZE((*fragment).contents) as usize,
                );
            } else if ob_type == STR_TYPE {
                let uni = unicode_to_str((*fragment).contents);
                if unlikely!(uni.is_none()) {
                    err!(SerializeError::InvalidStr)
                }
                buffer = uni.unwrap().as_bytes();
            } else {
                err!(SerializeError::InvalidFragment)
            }
        }
        serializer.serialize_bytes(buffer)
    }
}
