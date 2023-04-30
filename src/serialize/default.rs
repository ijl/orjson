// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::error::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, Serializer};

use std::ptr::NonNull;

pub struct DefaultSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DefaultSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
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

impl Serialize for DefaultSerializer {
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.default {
            Some(callable) => {
                if unlikely!(self.default_calls == RECURSION_LIMIT) {
                    err!(SerializeError::DefaultRecursionLimit)
                }
                let default_obj = ffi!(PyObject_CallFunctionObjArgs(
                    callable.as_ptr(),
                    self.ptr,
                    std::ptr::null_mut() as *mut pyo3_ffi::PyObject
                ));
                if unlikely!(default_obj.is_null()) {
                    err!(SerializeError::UnsupportedType(nonnull!(self.ptr)))
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
            None => err!(SerializeError::UnsupportedType(nonnull!(self.ptr))),
        }
    }
}
