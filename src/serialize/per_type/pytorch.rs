use core::ffi::c_char;
use crate::serialize::error::SerializeError;
use crate::serialize::per_type::{DefaultSerializer, NumpySerializer};
use crate::serialize::serializer::PyObjectSerializer;
use crate::typeref::{PYTORCH_TENSOR_TYPE};
use pyo3_ffi::*;
use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct PyTorchSerializer<'a> {
    previous: &'a PyObjectSerializer,
}

impl<'a> PyTorchSerializer<'a> {
    pub fn new(previous: &'a PyObjectSerializer) -> Self {
        Self { previous }
    }
}

#[cold]
pub fn is_pytorch_tensor(ob_type: *mut PyTypeObject) -> bool {
    unsafe { ob_type == PYTORCH_TENSOR_TYPE }
}

impl<'a> Serialize for PyTorchSerializer<'a> {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            // Get detach() method from tensor if it requires grad
            let detach_method = PyUnicode_InternFromString("detach\0".as_ptr() as *const c_char);
            let detached = PyObject_CallMethodObjArgs(self.previous.ptr, detach_method, std::ptr::null_mut::<pyo3_ffi::PyObject>());
            Py_DECREF(detach_method);

            // Get numpy() method from detached tensor
            let numpy_method = PyUnicode_InternFromString("numpy\0".as_ptr() as *const c_char);
            let numpy_array = if detached.is_null() {
                // If detach failed (tensor doesn't require grad), try numpy directly
                PyObject_CallMethodObjArgs(self.previous.ptr, numpy_method, std::ptr::null_mut::<pyo3_ffi::PyObject>())
            } else {
                let result = PyObject_CallMethodObjArgs(detached, numpy_method, std::ptr::null_mut::<pyo3_ffi::PyObject>());
                Py_DECREF(detached);
                result
            };
            Py_DECREF(numpy_method);

            if numpy_array.is_null() {
                PyErr_Clear();
                if self.previous.default.is_some() {
                    DefaultSerializer::new(self.previous).serialize(serializer)
                } else {
                    err!(SerializeError::PyTorchTensorConversion)
                }
            } else {
                // Create a new PyObjectSerializer for the numpy array
                let numpy_serializer = PyObjectSerializer {
                    ptr: numpy_array,
                    default: self.previous.default,
                    state: self.previous.state,
                };

                // Serialize using NumpySerializer
                let result = NumpySerializer::new(&numpy_serializer).serialize(serializer);
                Py_DECREF(numpy_array);
                result
            }
        }
    }
}