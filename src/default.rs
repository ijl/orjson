// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::encode::*;

use serde::ser::{Serialize, Serializer};
use std::ffi::CStr;

use std::ptr::NonNull;

macro_rules! obj_name {
    ($obj:ident) => {
        unsafe { CStr::from_ptr((*$obj).tp_name).to_string_lossy() }
    };
}

pub struct DefaultSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: u16,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl DefaultSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: u16,
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
                let obj_ptr = unsafe { (*self.ptr).ob_type };
                let default_obj = unsafe {
                    pyo3::ffi::PyObject_CallFunctionObjArgs(
                        callable.as_ptr(),
                        self.ptr,
                        std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
                    )
                };
                if default_obj.is_null() {
                    err!(format_args!(
                        "Type is not JSON serializable: {}",
                        obj_name!(obj_ptr)
                    ))
                } else if !ffi!(PyErr_Occurred()).is_null() {
                    err!(format_args!(
                        "Type raised exception in default function: {}",
                        obj_name!(obj_ptr)
                    ))
                } else {
                    let res = SerializePyObject::new(
                        default_obj,
                        None,
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
            None => {
                let obj_ptr = unsafe { (*self.ptr).ob_type };
                err!(format_args!(
                    "Type is not JSON serializable: {}",
                    obj_name!(obj_ptr)
                ))
            }
        }
    }
}
