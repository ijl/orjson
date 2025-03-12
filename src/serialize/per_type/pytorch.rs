use crate::serialize::error::SerializeError;
use crate::serialize::per_type::{DefaultSerializer, NumpySerializer};
use crate::serialize::serializer::PyObjectSerializer;
use core::ffi::c_char;
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

impl<'a> Serialize for PyTorchSerializer<'a> {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            let ptr = self.previous.ptr;

            // Approach: detach -> cpu -> numpy

            // Get detach() method from tensor if it requires grad
            let detach_method = PyUnicode_InternFromString("detach\0".as_ptr() as *const c_char);
            let detached =
                PyObject_CallMethodObjArgs(ptr, detach_method, std::ptr::null_mut::<PyObject>());
            Py_DECREF(detach_method);

            // Get cpu() method to ensure tensor is on CPU
            let cpu_method = PyUnicode_InternFromString("cpu\0".as_ptr() as *const c_char);
            let cpu_tensor = if detached.is_null() {
                PyObject_CallMethodObjArgs(ptr, cpu_method, std::ptr::null_mut::<PyObject>())
            } else {
                let result = PyObject_CallMethodObjArgs(
                    detached,
                    cpu_method,
                    std::ptr::null_mut::<PyObject>(),
                );
                Py_DECREF(detached);
                result
            };
            Py_DECREF(cpu_method);

            // Get numpy() method from CPU tensor
            let numpy_method = PyUnicode_InternFromString("numpy\0".as_ptr() as *const c_char);
            let numpy_array = if !cpu_tensor.is_null() {
                let result = PyObject_CallMethodObjArgs(
                    cpu_tensor,
                    numpy_method,
                    std::ptr::null_mut::<PyObject>(),
                );
                Py_DECREF(cpu_tensor);
                result
            } else {
                std::ptr::null_mut()
            };
            Py_DECREF(numpy_method);

            if numpy_array.is_null() {
                // If conversion fails, try default serializer or error
                PyErr_Clear();
                if self.previous.default.is_some() {
                    DefaultSerializer::new(self.previous).serialize(serializer)
                } else {
                    err!(SerializeError::PyTorchTensorConversion)
                }
            } else {
                // Create a PyObjectSerializer for the numpy array
                let numpy_serializer = PyObjectSerializer {
                    ptr: numpy_array,
                    default: self.previous.default,
                    state: self.previous.state,
                };

                // Use NumpySerializer directly for better performance
                // This avoids the unnecessary copy of data that tolist() would create
                let result = NumpySerializer::new(&numpy_serializer).serialize(serializer);
                Py_DECREF(numpy_array);
                result
            }
        }
    }
}
