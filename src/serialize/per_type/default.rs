// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::serialize::error::SerializeError;
use crate::serialize::serializer::PyObjectSerializer;

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct DefaultSerializer<'a> {
    previous: &'a PyObjectSerializer,
}

impl<'a> DefaultSerializer<'a> {
    pub fn new(previous: &'a PyObjectSerializer) -> Self {
        Self { previous: previous }
    }
}

impl<'a> Serialize for DefaultSerializer<'a> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.previous.default {
            Some(callable) => {
                if unlikely!(self.previous.state.default_calls_limit()) {
                    err!(SerializeError::DefaultRecursionLimit)
                }
                #[cfg(not(Py_3_10))]
                let default_obj = ffi!(PyObject_CallFunctionObjArgs(
                    callable.as_ptr(),
                    self.previous.ptr,
                    core::ptr::null_mut() as *mut pyo3_ffi::PyObject
                ));
                #[cfg(Py_3_10)]
                let default_obj = unsafe {
                    pyo3_ffi::PyObject_Vectorcall(
                        callable.as_ptr(),
                        core::ptr::addr_of!(self.previous.ptr),
                        pyo3_ffi::PyVectorcall_NARGS(1) as usize,
                        core::ptr::null_mut(),
                    )
                };
                if unlikely!(default_obj.is_null()) {
                    err!(SerializeError::UnsupportedType(nonnull!(self.previous.ptr)))
                } else {
                    let res = PyObjectSerializer::new(
                        default_obj,
                        self.previous.state.copy_for_default_call(),
                        self.previous.default,
                    )
                    .serialize(serializer);
                    ffi!(Py_DECREF(default_obj));
                    res
                }
            }
            None => err!(SerializeError::UnsupportedType(nonnull!(self.previous.ptr))),
        }
    }
}
