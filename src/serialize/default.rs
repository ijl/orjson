// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, Serializer};
use std::ffi::CStr;

use std::ptr::NonNull;

#[cold]
#[inline(never)]
fn format_err(ptr: *mut pyo3::ffi::PyObject) -> String {
    let name = unsafe { CStr::from_ptr((*ob_type!(ptr)).tp_name).to_string_lossy() };
    format_args!("Type is not JSON serializable: {}", name).to_string()
}

pub struct DefaultSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl DefaultSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        DefaultSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl<'p> Serialize for DefaultSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.default {
            Some(callable) => {
                if unlikely!(self.default_calls == RECURSION_LIMIT) {
                    err!("default serializer exceeds recursion limit")
                }
                let default_obj = ffi!(PyObject_CallFunctionObjArgs(
                    callable.as_ptr(),
                    self.ptr,
                    std::ptr::null_mut() as *mut pyo3::ffi::PyObject
                ));
                if unlikely!(default_obj.is_null()) {
                    err!(format_err(self.ptr))
                } else {
                    let res = PyObjectSerializer::new(
                        default_obj,
                        self.opts,
                        self.default_calls + 1,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer);
                    ffi!(Py_DECREF(default_obj));
                    res
                }
            }
            None => err!(format_err(self.ptr)),
        }
    }
}
